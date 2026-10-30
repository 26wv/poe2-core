//! The line prefix grammar and `parse_line` (docs/research/client-log.md "Line prefix", Unknown 1–5, Conflicting 2–3; ADRs 0008, 0012).

use std::fmt;
use std::str::FromStr;

use serde::{Serialize, Serializer};

use super::event::{Event, match_body};

/// One non-blank line of a client log: its prefix fields, when it has them, and its event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct LogLine {
    /// The line's date and time; present on prefixed lines and on the log-opened line.
    pub timestamp: Option<Timestamp>,
    /// The prefix fields after the time; present only when the line matches the full prefix grammar.
    pub prefix: Option<Prefix>,
    /// What the line means.
    pub event: Event,
}

/// A line's date and time as written, with no time zone and no calendar check.
///
/// Parsers read it as local time; the time zone is unconfirmed (ticket `prefix-field-meanings`).
/// It displays and serialises as `YYYY-MM-DDTHH:MM:SS`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Timestamp {
    /// The year.
    pub year: u16,
    /// The month, as written.
    pub month: u8,
    /// The day of the month, as written.
    pub day: u8,
    /// The hour (24-hour clock).
    pub hour: u8,
    /// The minute.
    pub minute: u8,
    /// The second.
    pub second: u8,
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Timestamp {
            year,
            month,
            day,
            hour,
            minute,
            second,
        } = self;
        write!(
            f,
            "{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}"
        )
    }
}

impl Serialize for Timestamp {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.collect_str(self)
    }
}

/// The prefix fields after a line's date and time: `<ms counter> <hex id> [<log level> <log source> <pid>]`.
///
/// Only confirmed meanings are stated here (ticket `prefix-field-meanings`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct Prefix {
    /// The decimal number after the time; it grows like a millisecond counter, from an unknown epoch.
    pub ms_counter: u64,
    /// The hex digits after the counter, as written; their meaning is unknown.
    pub hex_id: String,
    /// The first word inside the brackets (`INFO` and `DEBUG` are the values seen).
    pub log_level: String,
    /// The second word inside the brackets (`Client` is the only value seen).
    pub log_source: String,
    /// The number after the log source; one tool reads it as the process id (unconfirmed).
    pub pid: u32,
}

/// Parses one client log line; `None` only when the line is blank (ADR 0008).
///
/// One trailing `\n`, then one trailing `\r`, then one leading U+FEFF are removed first. A line
/// that doesn't match the full prefix grammar has no prefix fields and is [`Event::Unknown`].
pub fn parse_line(line: &str) -> Option<LogLine> {
    let line = line.strip_suffix('\n').unwrap_or(line);
    let line = line.strip_suffix('\r').unwrap_or(line);
    let text = line.strip_prefix('\u{feff}').unwrap_or(line);
    if text.is_empty() {
        return None;
    }
    let unknown = || Event::Unknown {
        raw: text.to_owned(),
    };
    if let Some((timestamp, rest)) = split_timestamp(text) {
        if rest == "***** LOG FILE OPENING *****" {
            return Some(LogLine {
                timestamp: Some(timestamp),
                prefix: None,
                event: Event::LogOpened,
            });
        }
        if let Some((prefix, body)) = split_prefix(rest) {
            let event = match_body(body).unwrap_or_else(unknown);
            return Some(LogLine {
                timestamp: Some(timestamp),
                prefix: Some(prefix),
                event,
            });
        }
    }
    Some(LogLine {
        timestamp: None,
        prefix: None,
        event: unknown(),
    })
}

/// Splits `dddd/dd/dd dd:dd:dd ` off the front of `text`.
fn split_timestamp(text: &str) -> Option<(Timestamp, &str)> {
    const SHAPE: &[u8; 20] = b"dddd/dd/dd dd:dd:dd ";
    let (head, rest) = text.split_at_checked(SHAPE.len())?;
    let fits = head.bytes().zip(SHAPE).all(|(b, &s)| {
        if s == b'd' {
            b.is_ascii_digit()
        } else {
            b == s
        }
    });
    if !fits {
        return None;
    }
    let field = |from: usize, to: usize| head.get(from..to).and_then(digits::<u8>);
    let timestamp = Timestamp {
        year: head.get(0..4).and_then(digits)?,
        month: field(5, 7)?,
        day: field(8, 10)?,
        hour: field(11, 13)?,
        minute: field(14, 16)?,
        second: field(17, 19)?,
    };
    Some((timestamp, rest))
}

/// Splits `<digits> <hex+> [<A-Z+> <non-space+> <digits>] ` off the front of `rest`.
fn split_prefix(rest: &str) -> Option<(Prefix, &str)> {
    let (ms_counter, rest) = rest.split_once(' ')?;
    let (hex_id, rest) = rest.split_once(' ')?;
    let (log_level, rest) = rest.strip_prefix('[')?.split_once(' ')?;
    let (log_source, rest) = rest.split_once(' ')?;
    let (pid, body) = rest.split_once("] ")?;
    let all = |s: &str, f: fn(char) -> bool| !s.is_empty() && s.chars().all(f);
    let fits = all(hex_id, |c| c.is_ascii_hexdigit())
        && all(log_level, |c| c.is_ascii_uppercase())
        && all(log_source, |c| !c.is_whitespace());
    if !fits {
        return None;
    }
    let prefix = Prefix {
        ms_counter: digits(ms_counter)?,
        hex_id: hex_id.to_owned(),
        log_level: log_level.to_owned(),
        log_source: log_source.to_owned(),
        pid: digits(pid)?,
    };
    Some((prefix, body))
}

/// Parses a non-empty run of ASCII digits; a sign, any other character or an overflow gives `None`.
fn digits<T: FromStr>(s: &str) -> Option<T> {
    if s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    s.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::log::Event;
    const P: &str = "2026/01/02 03:04:05 123456789 1a2b3c4d [INFO Client 4242] ";
    fn ts() -> Timestamp {
        Timestamp {
            year: 2026,
            month: 1,
            day: 2,
            hour: 3,
            minute: 4,
            second: 5,
        }
    }
    fn bare(l: &str) -> (Option<Timestamp>, Option<Prefix>, Event) {
        let g = parse_line(l).unwrap();
        (g.timestamp, g.prefix, g.event)
    }
    #[test]
    fn blank_lines_are_none() {
        for l in ["", "\n", "\r\n", "\u{feff}", "\u{feff}\r\n"] {
            assert_eq!(parse_line(l), None, "{l:?}");
        }
    }
    #[test]
    fn a_full_prefix_with_an_unknown_body() {
        let text = format!("{P}[WINDOW] Gained focus");
        let prefix = Prefix {
            ms_counter: 123456789,
            hex_id: "1a2b3c4d".into(),
            log_level: "INFO".into(),
            log_source: "Client".into(),
            pid: 4242,
        };
        assert_eq!(
            bare(&format!("{text}\r\n")),
            (Some(ts()), Some(prefix), Event::Unknown { raw: text })
        );
    }
    #[test]
    fn strips_one_newline_one_cr_and_one_bom() {
        assert_eq!(
            bare("abc\r\r\n").2,
            Event::Unknown {
                raw: "abc\r".into()
            }
        );
        assert_eq!(
            bare("\u{feff}\u{feff}x").2,
            Event::Unknown {
                raw: "\u{feff}x".into()
            }
        );
        assert!(bare(&format!("\u{feff}{P}x")).1.is_some());
    }
    #[test]
    fn log_opened_has_only_a_timestamp() {
        assert_eq!(
            bare("2026/01/02 03:04:05 ***** LOG FILE OPENING *****"),
            (Some(ts()), None, Event::LogOpened)
        );
    }
    #[test]
    fn prefix_variants() {
        let g = parse_line("2026/01/02 03:04:05 1 1A2B3C4 [DEBUG Client 7] ").unwrap();
        assert_eq!(
            (
                g.prefix.as_ref().unwrap().hex_id.as_str(),
                g.prefix.as_ref().unwrap().log_level.as_str()
            ),
            ("1A2B3C4", "DEBUG")
        );
        assert!(matches!(g.event, Event::Unknown { .. }));
    }
    #[test]
    fn lines_without_the_full_prefix_are_unknown_without_fields() {
        for l in [
            "hello",
            "@From TraderJoe: hi",
            "2026-01-02 03:04:05 1 a [INFO Client 1] x",
            "226/01/02 03:04:05 1 a [INFO Client 1] x",
            "2026/01/02 03:04:05 1 a [INFO Client] x",
            "2026/01/02 03:04:05 1 a [info Client 1] x",
            "2026/01/02 03:04:05 1 zz [INFO Client 1] x",
            "2026/01/02 03:04:05 +1 a [INFO Client 1] x",
            "2026/01/02 03:04:05 123456789012345678901 a [INFO Client 1] x",
            "2026/01/02 03:04:05 1 a [INFO Client 4294967296] x",
            "2026/01/02 03:04:05 1 a [INFO Client 1]x",
            "2026/01/02 03:04:05 1 a [INFO Client 1]",
            "2026/01/02 03:04:05 ***** LOG FILE OPENING ***** again",
        ] {
            assert_eq!(
                bare(l),
                (None, None, Event::Unknown { raw: l.into() }),
                "{l:?}"
            );
        }
    }
    #[test]
    fn json_shape_is_exact() {
        let g = parse_line(&format!("{P}[WINDOW] Gained focus")).unwrap();
        assert_eq!(
            serde_json::to_string(&g).unwrap(),
            r#"{"timestamp":"2026-01-02T03:04:05","prefix":{"ms_counter":123456789,"hex_id":"1a2b3c4d","log_level":"INFO","log_source":"Client","pid":4242},"event":{"type":"Unknown","raw":"2026/01/02 03:04:05 123456789 1a2b3c4d [INFO Client 4242] [WINDOW] Gained focus"}}"#
        );
        assert_eq!(
            serde_json::to_string(&parse_line("hello").unwrap()).unwrap(),
            r#"{"timestamp":null,"prefix":null,"event":{"type":"Unknown","raw":"hello"}}"#
        );
        assert_eq!(
            serde_json::to_string(&Event::LogOpened).unwrap(),
            r#"{"type":"LogOpened"}"#
        );
        assert_eq!(ts().to_string(), "2026-01-02T03:04:05");
    }
}
