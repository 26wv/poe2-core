//! KeyValues reader for library lists and app manifests (docs/research/windows-paths.md Q2, docs/research/linux-paths.md Q2; ADRs 0005, 0018).

use std::iter::Peekable;
use std::str::Chars;

/// Deeper nesting makes the whole file malformed.
const MAX_DEPTH: usize = 32;

enum Value {
    Str(String),
    Obj(Vec<(String, Value)>),
}

enum Token {
    Open,
    Close,
    Str(String),
    End,
}

struct Tokens<'a>(Peekable<Chars<'a>>);

impl Tokens<'_> {
    /// The next token, or `None` for an unterminated quoted string.
    fn next(&mut self) -> Option<Token> {
        loop {
            let Some(c) = self.0.next() else {
                return Some(Token::End);
            };
            match c {
                '{' => return Some(Token::Open),
                '}' => return Some(Token::Close),
                '"' => return self.quoted().map(Token::Str),
                '/' if self.0.next_if_eq(&'/').is_some() => {
                    self.0.find(|&c| c == '\n');
                }
                c if c.is_ascii_whitespace() => {}
                c => {
                    let mut s = String::from(c);
                    while let Some(c) = self
                        .0
                        .next_if(|&c| !(c.is_ascii_whitespace() || matches!(c, '{' | '}' | '"')))
                    {
                        s.push(c);
                    }
                    return Some(Token::Str(s));
                }
            }
        }
    }

    fn quoted(&mut self) -> Option<String> {
        let mut s = String::new();
        loop {
            match self.0.next()? {
                '"' => return Some(s),
                '\\' => match self.0.next()? {
                    '\\' => s.push('\\'),
                    '"' => s.push('"'),
                    'n' => s.push('\n'),
                    't' => s.push('\t'),
                    c => {
                        s.push('\\');
                        s.push(c);
                    }
                },
                c => s.push(c),
            }
        }
    }
}

/// The file's top-level pairs, or `None` if it is malformed. Nesting uses an explicit stack, not recursion.
fn parse(text: &str) -> Option<Vec<(String, Value)>> {
    let mut tokens = Tokens(text.chars().peekable());
    let mut open: Vec<(String, Vec<(String, Value)>)> = Vec::new();
    let mut pairs = Vec::new();
    loop {
        let key = match tokens.next()? {
            Token::Str(key) => key,
            Token::Close => {
                let (key, parent) = open.pop()?;
                let obj = std::mem::replace(&mut pairs, parent);
                pairs.push((key, Value::Obj(obj)));
                continue;
            }
            Token::End if open.is_empty() => return Some(pairs),
            Token::End | Token::Open => return None,
        };
        match tokens.next()? {
            Token::Str(value) => pairs.push((key, Value::Str(value))),
            Token::Open if open.len() < MAX_DEPTH => open.push((key, std::mem::take(&mut pairs))),
            Token::Open | Token::Close | Token::End => return None,
        }
    }
}

/// The first value under `key`, ignoring ASCII case.
fn get<'a>(pairs: &'a [(String, Value)], key: &str) -> Option<&'a Value> {
    pairs
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case(key))
        .map(|(_, v)| v)
}

/// Library list paths in file order; empty if malformed.
pub(crate) fn library_paths(text: &str) -> Vec<String> {
    let root = parse(text).unwrap_or_default();
    let Some(Value::Obj(libraries)) = get(&root, "libraryfolders") else {
        return Vec::new();
    };
    libraries
        .iter()
        .filter(|(key, _)| !key.is_empty() && key.bytes().all(|b| b.is_ascii_digit()))
        .filter_map(|(_, library)| match library {
            Value::Str(legacy) => Some(legacy.clone()),
            Value::Obj(library) => {
                let unmounted = get(library, "mounted")
                    .is_some_and(|m| !matches!(m, Value::Str(m) if m == "1"));
                match get(library, "path") {
                    Some(Value::Str(path)) if !unmounted => Some(path.clone()),
                    _ => None,
                }
            }
        })
        .collect()
}

/// The app manifest's `AppState.installdir`.
pub(crate) fn installdir(text: &str) -> Option<String> {
    let root = parse(text)?;
    let Some(Value::Obj(app)) = get(&root, "appstate") else {
        return None;
    };
    match get(app, "installdir")? {
        Value::Str(dir) => Some(dir.clone()),
        Value::Obj(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    const CURRENT: &str = "\"libraryfolders\"\n{\n\t\"contentstatsid\"\t\t\"-1111111111111111111\"\n\t\"0\"\n\t{\n\t\t\"path\"\t\t\"C:\\\\Program Files (x86)\\\\Steam\"\n\t\t\"apps\"\n\t\t{\n\t\t\t\"2694490\"\t\t\"150469730160\"\n\t\t}\n\t}\n\t\"1\"\n\t{\n\t\t\"path\"\t\t\"D:\\\\SteamLibrary\"\n\t\t\"label\"\t\t\"say \\\"hi\\\"\"\n\t}\n}\n";
    const LEGACY: &str = "\"LibraryFolders\" { \"TimeNextStatsReport\" \"1234567890\" \"1\" \"D:\\\\SteamLibrary\" \"2\" { \"path\" \"E:\\\\Games\\\\Steam\" \"mounted\" \"1\" } \"3\" { \"path\" \"F:\\\\Offline\" \"mounted\" \"0\" } }";
    #[test]
    fn reads_both_library_list_layouts() {
        assert_eq!(
            library_paths(CURRENT),
            [r"C:\Program Files (x86)\Steam", r"D:\SteamLibrary"]
        );
        assert_eq!(
            library_paths(LEGACY),
            [r"D:\SteamLibrary", r"E:\Games\Steam"]
        );
        assert_eq!(
            library_paths("// c\n\"libraryfolders\" { 0 { path /mnt/games } }"),
            ["/mnt/games"]
        );
    }
    #[test]
    fn reads_installdir_ignoring_key_case() {
        assert_eq!(
            installdir("\"AppState\" { \"appid\" \"2694490\" \"InstallDir\" \"Path of Exile 2\" }")
                .as_deref(),
            Some("Path of Exile 2")
        );
        assert_eq!(
            installdir("\"appstate\" { \"installdir\" \"PoE2\" }").as_deref(),
            Some("PoE2")
        );
        assert_eq!(installdir("\"AppState\" { \"name\" \"x\" }"), None);
    }
    #[test]
    fn malformed_and_hostile_input_reads_as_nothing() {
        for t in [
            "",
            "\"libraryfolders\" {",
            "}",
            "\"a\"",
            "\"libraryfolders\" { \"0\" { \"path\" } }",
            "\"unterminated",
        ] {
            assert!(library_paths(t).is_empty(), "{t:?}");
            assert_eq!(installdir(t), None, "{t:?}");
        }
        let deep = format!(
            "\"libraryfolders\" {}{}",
            "{".repeat(100_000),
            "}".repeat(100_000)
        );
        assert!(library_paths(&deep).is_empty());
    }
    #[test]
    fn nesting_past_32_levels_is_malformed() {
        let nested = |n: usize| format!("{}\"x\" \"y\"{}", "\"k\" { ".repeat(n), "} ".repeat(n));
        assert!(parse(&nested(32)).is_some());
        assert!(parse(&nested(33)).is_none());
        assert!(parse(&nested(100_000)).is_none());
    }
    #[test]
    fn unescapes_only_the_four_keyvalues_escapes() {
        let text = r#""AppState" { "installdir" "a\\b\"c\nd\te\qf" }"#;
        assert_eq!(installdir(text).as_deref(), Some("a\\b\"c\nd\te\\qf"));
    }
    proptest! {
        #[test] fn never_panics(s in any::<String>()) { let _ = (library_paths(&s), installdir(&s)); }
        #[test] fn never_panics_on_token_soup(parts in proptest::collection::vec(prop_oneof![Just("\""), Just("{"), Just("}"), Just("\\"), Just("//"), Just("\n"), Just("libraryfolders"), Just("path"), Just("0"), Just("mounted")], 0..64)) {
            let s = parts.concat();
            let _ = (library_paths(&s), installdir(&s));
        }
    }
}
