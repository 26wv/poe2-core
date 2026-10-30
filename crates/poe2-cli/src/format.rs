//! Event-line output for `parse` and `tail` (ADRs 0012 and 0016).

use std::io::{self, Write};

use poe2_core::log::LogLine;

/// Writes `line` as compact JSON, or as `<timestamp or -> <type> <key>=<JSON value> …`, then `\n`.
pub(crate) fn write_line(out: &mut dyn Write, line: &LogLine, json: bool) -> io::Result<()> {
    if json {
        return writeln!(out, "{}", serde_json::to_string(line)?);
    }
    match &line.timestamp {
        Some(t) => write!(out, "{t}")?,
        None => write!(out, "-")?,
    }
    let event = serde_json::to_value(&line.event)?;
    write!(out, " {}", event["type"].as_str().unwrap_or_default())?;
    for (key, value) in event.as_object().into_iter().flatten() {
        if key != "type" {
            write!(out, " {key}={value}")?;
        }
    }
    writeln!(out)
}

#[cfg(test)]
mod tests {
    #[test]
    fn human_and_json_lines() {
        use poe2_core::log::parse_line;
        let mut out = Vec::new();
        for l in [
            "2026/01/02 03:04:05 123456789 1a2b3c4d [INFO Client 4242] : ExileOne (Smith of Kitava) is now level 12",
            "hello",
            "2026/01/02 03:04:05 ***** LOG FILE OPENING *****",
        ] {
            super::write_line(&mut out, &parse_line(l).unwrap(), false).unwrap();
        }
        assert_eq!(
            String::from_utf8(out).unwrap(),
            "2026-01-02T03:04:05 LevelUp class=\"Smith of Kitava\" level=12 name=\"ExileOne\"\n- Unknown raw=\"hello\"\n2026-01-02T03:04:05 LogOpened\n"
        );
        let mut json = Vec::new();
        let line = parse_line("hello").unwrap();
        super::write_line(&mut json, &line, true).unwrap();
        assert_eq!(
            String::from_utf8(json).unwrap(),
            serde_json::to_string(&line).unwrap() + "\n"
        );
    }
}
