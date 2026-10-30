//! Scoped YAML scalar reader for Lutris game files and Bottles' `placeholder.yml` and `data.yml` (docs/research/linux-paths.md Q6, Q7, Q9; ADRs 0005, 0018).

use std::str::Chars;

/// The scalar at `path` (`[top]` or `[top, child]`), decoded. Only the block under the requested
/// top-level key is searched, so `script: game: prefix:` never matches `["game", "prefix"]`.
pub(crate) fn scalar(text: &str, path: &[&str]) -> Option<String> {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let mut lines = text.split('\n').map(|l| l.strip_suffix('\r').unwrap_or(l));
    let (top, child) = match path {
        [top] => (*top, None),
        [top, child] => (*top, Some(*child)),
        _ => return None,
    };
    let rest = lines.find_map(|l| key_value(l, 0, top))?;
    let Some(child) = child else {
        return value(rest, 0, lines);
    };
    let rest = rest.trim_start_matches([' ', '\t']);
    if !(rest.is_empty() || rest.starts_with('#')) {
        return None; // `top` holds a scalar or a flow collection, not a block mapping
    }
    let mut child_indent = None;
    loop {
        let line = lines.next()?;
        let (indent, body) = split_indent(line);
        if body.trim_start_matches('\t').is_empty() || body.starts_with('#') {
            continue;
        }
        if indent == 0 {
            return None; // the next top-level key: `top`'s block has ended
        }
        let want = *child_indent.get_or_insert(indent);
        if let Some(rest) = key_value(line, want, child) {
            return value(rest, want, lines);
        }
    }
}

/// The count of leading spaces, and the rest of `line`.
fn split_indent(line: &str) -> (usize, &str) {
    let body = line.trim_start_matches(' ');
    (line.len().saturating_sub(body.len()), body)
}

/// What follows `key:` when `line` is that key at exactly `indent`.
fn key_value<'a>(line: &'a str, indent: usize, key: &str) -> Option<&'a str> {
    let (n, body) = split_indent(line);
    let rest = body.strip_prefix(key)?.strip_prefix(':')?;
    (n == indent && (rest.is_empty() || rest.starts_with([' ', '\t']))).then_some(rest)
}

/// Decodes the scalar starting at `first` with its continuation lines: those indented more than
/// its key, left-trimmed, with blank ones kept as empty lines.
fn value<'a>(first: &str, indent: usize, lines: impl Iterator<Item = &'a str>) -> Option<String> {
    let mut raw = first.trim_start_matches([' ', '\t']).to_owned();
    if raw.is_empty() {
        return None;
    }
    let mut blanks = 0;
    for line in lines {
        let (n, body) = split_indent(line);
        let body = body.trim_start_matches([' ', '\t']);
        if body.is_empty() {
            blanks += 1;
            continue;
        }
        if n <= indent {
            break;
        }
        for _ in 0..=blanks {
            raw.push('\n');
        }
        blanks = 0;
        raw.push_str(body);
    }
    if let Some(s) = raw.strip_prefix('\'') {
        quoted(s, '\'')
    } else if let Some(s) = raw.strip_prefix('"') {
        quoted(s, '"')
    } else if raw.starts_with(['[', '{', '|', '>', '&', '*', '!', '%', '@', '`']) {
        None // flow collections, block scalars, anchors, aliases, tags and reserved indicators
    } else {
        plain(&raw)
    }
}

/// A plain scalar: lines fold with one space (a blank line gives `\n`), and ` #` starts a comment.
fn plain(raw: &str) -> Option<String> {
    let mut out = String::new();
    let mut blanks = 0;
    for line in raw.split('\n') {
        let comment = line.match_indices('#').map(|(i, _)| i).find(|&i| {
            line.get(..i)
                .is_some_and(|p| p.is_empty() || p.ends_with([' ', '\t']))
        });
        let text = line.get(..comment.unwrap_or(line.len()))?;
        let text = text.trim_end_matches([' ', '\t']);
        if text.is_empty() && comment.is_none() {
            blanks += 1;
            continue;
        }
        if !out.is_empty() && !text.is_empty() {
            push_breaks(&mut out, blanks);
        }
        blanks = 0;
        out.push_str(text);
        if comment.is_some() {
            break;
        }
    }
    (!out.is_empty()).then_some(out)
}

/// A line fold: a single line break becomes a space, and each blank line after it a `\n`.
fn push_breaks(out: &mut String, blanks: usize) {
    if blanks == 0 {
        out.push(' ');
    }
    for _ in 0..blanks {
        out.push('\n');
    }
}

/// The rest of a quoted scalar after its opening `q` (`'` or `"`), decoded.
fn quoted(s: &str, q: char) -> Option<String> {
    let mut out = String::new();
    let mut chars = s.chars();
    // Line breaks since the last character, whether they follow a `\` line join, and how much of
    // `out` ends in an escape, so trimming whitespace before a break keeps an escaped space or tab.
    let (mut breaks, mut joined, mut escaped) = (0usize, false, 0);
    loop {
        let c = chars.next()?;
        if c == '\n' {
            if breaks == 0 && !joined {
                // Scan only past the last escape, so each character is trimmed at most once.
                let keep = out
                    .get(escaped..)
                    .map_or(0, |t| t.trim_end_matches([' ', '\t']).len());
                out.truncate(escaped.saturating_add(keep));
            }
            breaks += 1;
            continue;
        }
        if joined {
            (0..breaks).for_each(|_| out.push('\n'));
        } else if breaks > 0 {
            push_breaks(&mut out, breaks.saturating_sub(1));
        }
        (breaks, joined) = (0, false);
        match c {
            '\'' if q == '\'' => {
                if !chars.as_str().starts_with('\'') {
                    return closed(chars.as_str(), out);
                }
                chars.next();
                out.push('\'');
            }
            '"' if q == '"' => return closed(chars.as_str(), out),
            '\\' if q == '"' => {
                let e = chars.next()?;
                if e == '\n' {
                    joined = true;
                    continue;
                }
                out.push(escape(e, &mut chars)?);
                escaped = out.len();
            }
            c => out.push(c),
        }
    }
}

/// `out`, when nothing but a comment follows the closing quote and `out` is not empty.
fn closed(rest: &str, out: String) -> Option<String> {
    let r = rest.trim_start_matches([' ', '\t']);
    let tail_ok =
        !r.contains('\n') && (r.is_empty() || (r.starts_with('#') && r.len() < rest.len()));
    (tail_ok && !out.is_empty()).then_some(out)
}

/// The character a double-quoted escape stands for, given the character after its `\`.
fn escape(e: char, chars: &mut Chars) -> Option<char> {
    Some(match e {
        '\\' => '\\',
        '"' => '"',
        '/' => '/',
        'n' => '\n',
        't' => '\t',
        ' ' => ' ',
        'x' => hex(chars, 2)?,
        'u' => hex(chars, 4)?,
        'U' => hex(chars, 8)?,
        _ => return None,
    })
}

/// The character named by the next `len` hex digits; `None` for a bad digit or a surrogate.
fn hex(chars: &mut Chars, len: usize) -> Option<char> {
    let (digits, tail) = chars.as_str().split_at_checked(len)?;
    if !digits.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    *chars = tail.chars();
    char::from_u32(u32::from_str_radix(digits, 16).ok()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    const PLAIN: &str = include_str!("fixtures/lutris_plain.yml");
    const FOLDED: &str = include_str!("fixtures/lutris_folded.yml");
    const UNICODE: &str = include_str!("fixtures/lutris_unicode.yml");
    #[test]
    fn fixtures_have_the_shapes_pyyaml_writes() {
        assert!(
            FOLDED.lines().count() >= 3,
            "prefix should fold onto a continuation line"
        );
        assert!(
            UNICODE.contains("\\x") && UNICODE.contains("\\\n"),
            "expected escapes and a \\ continuation"
        );
        assert!(include_str!("fixtures/lutris_type_like.yml").contains("'yes'"));
        assert!(include_str!("fixtures/lutris_quoted.yml").contains("''"));
    }
    #[test]
    fn reads_lutris_game_prefix_in_every_style() {
        let p = |t: &str| scalar(t, &["game", "prefix"]);
        assert_eq!(
            p(PLAIN).as_deref(),
            Some("/home/<user>/Games/path-of-exile-2")
        );
        assert_eq!(
            p(FOLDED).as_deref(),
            Some(
                "/home/<user>/Games/a folder name long enough that PyYAML folds this plain scalar onto a second line/path-of-exile-2"
            )
        );
        assert_eq!(
            p(UNICODE).as_deref(),
            Some(
                "/home/<user>/Jeux/Éditions spéciales/préfixe — Path of Exile 2 with enough extra words to wrap the line"
            )
        );
        assert_eq!(
            p(include_str!("fixtures/lutris_quoted.yml")).as_deref(),
            Some("/home/<user>/Games/PoE #2's prefix")
        );
        assert_eq!(
            p(include_str!("fixtures/lutris_type_like.yml")).as_deref(),
            Some("yes")
        );
        assert_eq!(p(include_str!("fixtures/lutris_script_only.yml")), None);
    }
    #[test]
    fn reads_bottles_top_level_values() {
        assert_eq!(
            scalar(include_str!("fixtures/bottles_placeholder.yml"), &["Path"]).as_deref(),
            Some("/mnt/games/bottles/PoE2")
        );
        assert_eq!(
            scalar(
                include_str!("fixtures/bottles_data.yml"),
                &["custom_bottles_path"]
            )
            .as_deref(),
            Some("/mnt/games/bottles")
        );
    }
    #[test]
    fn crlf_comments_and_missing_values() {
        assert_eq!(
            scalar("game:\r\n  prefix: /x # note\r\n", &["game", "prefix"]).as_deref(),
            Some("/x")
        );
        for t in [
            "",
            "game: [",
            "game:\n  prefix:\n",
            "prefix: /top\n",
            "game:\n  prefix: \"unterminated\n",
        ] {
            assert_eq!(scalar(t, &["game", "prefix"]), None, "{t:?}");
        }
    }
    #[test]
    fn escaped_space_lines_decode_in_linear_time() {
        // About 1 MiB (READ_CAP). Under a trim that rescans `out` at every break this takes minutes.
        let n = 200_000;
        let text = format!("Path: \"\\ {}\n  x\"\n", "\n  \\ ".repeat(n - 1));
        let want = format!("{}x", " ".repeat(2 * n));
        assert_eq!(scalar(&text, &["Path"]), Some(want));
    }
    proptest! {
        #[test]
        fn never_panics(s in any::<String>()) { let _ = (scalar(&s, &["game", "prefix"]), scalar(&s, &["Path"])); }
    }
}
