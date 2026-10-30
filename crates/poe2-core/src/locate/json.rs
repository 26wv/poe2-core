//! JSON token scanner for Epic manifests and Heroic configs (docs/research/windows-paths.md Q4, docs/research/linux-paths.md Q5, Q9; ADRs 0005, 0018).

use std::str::Chars;

enum Token<'a> {
    Str(String),
    Colon,
    /// `{ } [ ] ,`
    Punct,
    /// A literal or number, unchecked.
    Lit(&'a str),
    End,
}

/// Scans tokens left to right with no stack, so nesting depth costs nothing.
struct Scanner<'a>(Chars<'a>);

fn is_space(c: char) -> bool {
    matches!(c, ' ' | '\t' | '\n' | '\r')
}

impl<'a> Scanner<'a> {
    /// The next token, or `None` for a malformed string.
    fn next(&mut self) -> Option<Token<'a>> {
        let rest = self.0.as_str().trim_start_matches(is_space);
        self.0 = rest.chars();
        let Some(c) = self.0.next() else {
            return Some(Token::End);
        };
        Some(match c {
            '"' => Token::Str(self.string()?),
            ':' => Token::Colon,
            '{' | '}' | '[' | ']' | ',' => Token::Punct,
            _ => {
                let end = rest
                    .find(|c| is_space(c) || "{}[]:,\"".contains(c))
                    .unwrap_or(rest.len());
                let (lit, tail) = rest.split_at_checked(end)?;
                self.0 = tail.chars();
                Token::Lit(lit)
            }
        })
    }

    /// The rest of a string after its opening quote, unescaped.
    fn string(&mut self) -> Option<String> {
        let mut s = String::new();
        loop {
            match self.0.next()? {
                '"' => return Some(s),
                '\\' => s.push(match self.0.next()? {
                    '"' => '"',
                    '\\' => '\\',
                    '/' => '/',
                    'b' => '\u{8}',
                    'f' => '\u{c}',
                    'n' => '\n',
                    'r' => '\r',
                    't' => '\t',
                    'u' => self.unicode()?,
                    _ => return None,
                }),
                c => s.push(c),
            }
        }
    }

    /// The character after `\u`, joining a surrogate pair; a lone surrogate becomes U+FFFD.
    fn unicode(&mut self) -> Option<char> {
        let hi = self.hex4()?;
        if let 0xD800..0xDC00 = hi
            && let Some(rest) = self.0.as_str().strip_prefix("\\u")
        {
            let mut ahead = Scanner(rest.chars());
            if let Some(lo @ 0xDC00..0xE000) = ahead.hex4() {
                *self = ahead;
                // Both ranges are checked above, so this cannot overflow.
                return char::from_u32(0x10000 + ((hi - 0xD800) << 10) + (lo - 0xDC00));
            }
        }
        Some(char::from_u32(hi).unwrap_or('\u{fffd}'))
    }

    fn hex4(&mut self) -> Option<u32> {
        let rest = self.0.as_str();
        let (digits, tail) = rest.split_at_checked(4)?;
        if !digits.bytes().all(|b| b.is_ascii_hexdigit()) {
            return None;
        }
        self.0 = tail.chars();
        u32::from_str_radix(digits, 16).ok()
    }
}

/// The value token after the first member named exactly `key`, at any depth.
fn value<'a>(text: &'a str, key: &str) -> Option<Token<'a>> {
    let mut scanner = Scanner(text.strip_prefix('\u{feff}').unwrap_or(text).chars());
    let mut prev = Token::End;
    loop {
        let token = scanner.next()?;
        match (&prev, &token) {
            (_, Token::End) => return None,
            (Token::Str(k), Token::Colon) if k == key => return scanner.next(),
            _ => prev = token,
        }
    }
}

/// The first member named exactly `key`, if its value is a string.
pub(crate) fn string_value(text: &str, key: &str) -> Option<String> {
    match value(text, key)? {
        Token::Str(s) => Some(s),
        _ => None,
    }
}

/// The first member named exactly `key`, if its value is `true` or `false`.
pub(crate) fn bool_value(text: &str, key: &str) -> Option<bool> {
    match value(text, key)? {
        Token::Lit("true") => Some(true),
        Token::Lit("false") => Some(false),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    const ITEM: &str = r#"{ "FormatVersion": 0, "AppName": "57d39134358849b1b765ee649ea6a6e9", "DisplayName": "Path of Exile 2",
  "CatalogNamespace": "58a18be6bbba41a0bf52b014bb81d33b", "InstallLocation": "D:\\Epic Games\\PathOfExile2",
  "bIsIncompleteInstall": false, "Nested": { "InstallLocation": "ignored" } }"#;
    #[test]
    fn reads_epic_manifest_fields() {
        assert_eq!(
            string_value(ITEM, "InstallLocation").as_deref(),
            Some(r"D:\Epic Games\PathOfExile2")
        );
        assert_eq!(
            string_value(ITEM, "CatalogNamespace").as_deref(),
            Some("58a18be6bbba41a0bf52b014bb81d33b")
        );
        assert_eq!(bool_value(ITEM, "bIsIncompleteInstall"), Some(false));
        assert_eq!(string_value(ITEM, "installlocation"), None); // exact key match only
    }
    #[test]
    fn values_and_array_items_are_not_keys() {
        assert_eq!(
            string_value(r#"{"a": "winePrefix", "winePrefix": "/x"}"#, "winePrefix").as_deref(),
            Some("/x")
        );
        assert_eq!(
            string_value(
                r#"{"a": ["winePrefix", 1], "b": {"winePrefix": "/y"}}"#,
                "winePrefix"
            )
            .as_deref(),
            Some("/y")
        );
        assert_eq!(
            string_value(
                r#"{"s": "\"winePrefix\": \"/no\"", "winePrefix": "/z"}"#,
                "winePrefix"
            )
            .as_deref(),
            Some("/z")
        );
    }
    #[test]
    fn unescapes_everything_json_allows() {
        assert_eq!(
            string_value(
                r#"{"p": "C:\\G\/x\n\t\"q\" \u00e9 \ud83d\ude00 \ud800"}"#,
                "p"
            )
            .as_deref(),
            Some("C:\\G/x\n\t\"q\" é 😀 \u{fffd}")
        );
    }
    #[test]
    fn heroic_games_config_shape() {
        let t = "{\n  \"abc123\": {\n    \"winePrefix\": \"/home/<user>/Games/Heroic/Prefixes/Path of Exile 2\",\n    \"wineVersion\": {\n      \"type\": \"proton\"\n    }\n  },\n  \"version\": \"v0\"\n}";
        assert_eq!(
            string_value(t, "winePrefix").as_deref(),
            Some("/home/<user>/Games/Heroic/Prefixes/Path of Exile 2")
        );
    }
    #[test]
    fn wrong_types_and_malformed_input_are_none() {
        assert_eq!(string_value(r#"{"k": 1}"#, "k"), None);
        assert_eq!(bool_value(r#"{"k": "true"}"#, "k"), None);
        assert_eq!(bool_value(r#"{"k": true}"#, "k"), Some(true));
        for t in [
            r#"{"k": "unterminated"#,
            r#"{"k" "x"}"#,
            r#"{"k": "\q"}"#,
            r#"{"k": "\u12"}"#,
            "",
        ] {
            assert_eq!(string_value(t, "k"), None, "{t}");
        }
    }
    #[test]
    fn bom_minified_and_deep_nesting() {
        assert_eq!(
            string_value("\u{feff}{\"k\":\"v\"}", "k").as_deref(),
            Some("v")
        );
        assert_eq!(bool_value("{\"a\":[1,{}],\"k\":true}", "k"), Some(true));
        assert_eq!(bool_value("{\"k\"\n\t:\r\nfalse\n}", "k"), Some(false));
        assert_eq!(
            string_value(r#"{"p": "\ud800\u0041 \udc00"}"#, "p").as_deref(),
            Some("\u{fffd}A \u{fffd}")
        );
        let deep = format!(
            "{}\"k\": \"v\"{}",
            "[{".repeat(100_000),
            "}]".repeat(100_000)
        );
        assert_eq!(string_value(&deep, "k").as_deref(), Some("v"));
    }
    proptest! {
        #[test] fn never_panics(s in any::<String>(), k in "[a-zA-Z]{1,12}") { let _ = (string_value(&s, &k), bool_value(&s, &k)); }
    }
}
