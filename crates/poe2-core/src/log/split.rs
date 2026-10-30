//! The bounded line splitter (docs/research/client-log.md "File properties"; ADRs 0008, 0010, 0014).

use super::{LogLine, parse_line};

/// The line cap: a longer line is cut here and the rest skipped to the next `\n` (ADR 0014).
pub(crate) const MAX_LINE_BYTES: usize = 64 * 1024;
/// The read chunk size for log files (ADR 0014).
pub(crate) const CHUNK: usize = 64 * 1024;

/// Splits a byte stream into lines at `\n`, holding at most `cap` bytes of the current line.
pub(crate) struct Splitter {
    cap: usize,
    held: Vec<u8>,
    skipping: bool,
}

impl Splitter {
    pub(crate) fn new() -> Splitter {
        Splitter {
            cap: MAX_LINE_BYTES,
            held: Vec::new(),
            skipping: false,
        }
    }

    #[cfg(test)]
    pub(crate) fn with_cap(cap: usize) -> Splitter {
        Splitter {
            cap,
            ..Splitter::new()
        }
    }

    /// Calls `emit` with each complete line in `bytes`, without its `\n`; `\r` stays for `parse_line`.
    pub(crate) fn push(&mut self, bytes: &[u8], mut emit: impl FnMut(&[u8])) {
        let mut parts = bytes.split(|b| *b == b'\n');
        let mut part = parts.next().unwrap_or_default();
        for next in parts {
            self.hold(part, &mut emit);
            if !self.skipping {
                emit(&self.held);
            }
            self.held.clear();
            self.skipping = false;
            part = next;
        }
        self.hold(part, &mut emit);
    }

    /// Adds `part` to the held line; past the cap, emits the first `cap` bytes and starts skipping.
    fn hold(&mut self, part: &[u8], emit: &mut impl FnMut(&[u8])) {
        if self.skipping {
            return;
        }
        let room = self.cap.saturating_sub(self.held.len());
        match part.get(..room) {
            Some(kept) if kept.len() < part.len() => {
                self.held.extend_from_slice(kept);
                emit(&self.held);
                self.held.clear();
                self.skipping = true;
            }
            _ => self.held.extend_from_slice(part),
        }
    }

    /// Discards everything up to and including the next `\n`.
    pub(crate) fn skip_to_next_line(&mut self) {
        self.held.clear();
        self.skipping = true;
    }

    /// Returns the held partial line, if any, and empties it; `None` while skipping a long line's rest.
    pub(crate) fn take_partial(&mut self) -> Option<Vec<u8>> {
        (!self.held.is_empty()).then(|| std::mem::take(&mut self.held))
    }
}

/// Decodes one line as lossy UTF-8 and parses it; blank lines give `None`.
pub(crate) fn decode(line: &[u8]) -> Option<LogLine> {
    parse_line(&String::from_utf8_lossy(line))
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    fn lines(s: &mut Splitter, chunks: &[&[u8]]) -> Vec<String> {
        let mut out = Vec::new();
        for c in chunks {
            s.push(c, |l| out.push(String::from_utf8_lossy(l).into_owned()));
        }
        out
    }
    #[test]
    fn splits_across_chunks_and_keeps_cr() {
        assert_eq!(
            lines(&mut Splitter::with_cap(8), &[b"ab\ncd", b"e\r\n\n"]),
            ["ab", "cde\r", ""]
        );
    }
    #[test]
    fn holds_and_takes_the_partial_line() {
        let mut s = Splitter::with_cap(8);
        assert!(lines(&mut s, &[b"abc"]).is_empty());
        assert_eq!(s.take_partial(), Some(b"abc".to_vec()));
        assert_eq!(s.take_partial(), None);
    }
    #[test]
    fn cuts_long_lines_at_the_cap() {
        assert_eq!(
            lines(&mut Splitter::with_cap(8), &[b"0123456789\nxy\n"]),
            ["01234567", "xy"]
        );
        assert_eq!(
            lines(&mut Splitter::with_cap(8), &[b"01234567\n"]),
            ["01234567"]
        );
        let mut s = Splitter::with_cap(8);
        assert_eq!(lines(&mut s, &[b"0123", b"456789ab", b"cd"]), ["01234567"]);
        assert_eq!(s.take_partial(), None);
        assert_eq!(lines(&mut s, &[b"ef\nz\n"]), ["z"]);
    }
    #[test]
    fn skips_to_the_next_line() {
        let mut s = Splitter::with_cap(8);
        s.skip_to_next_line();
        assert_eq!(lines(&mut s, &[b"rest\nnext\n"]), ["next"]);
    }
    #[test]
    fn decode_is_lossy_and_drops_blank_lines() {
        assert_eq!(
            decode(b"a\xffb").map(|l| l.event),
            Some(crate::log::Event::Unknown {
                raw: "a\u{fffd}b".into()
            })
        );
        assert_eq!((decode(b""), decode(b"\r")), (None, None));
    }
    proptest! {
        #[test]
        fn never_panics_and_respects_the_cap(chunks in proptest::collection::vec(proptest::collection::vec(any::<u8>(), 0..64), 0..32), cap in 1usize..16) {
            let mut s = Splitter::with_cap(cap);
            for c in &chunks { s.push(c, |l| assert!(l.len() <= cap)); }
            let _ = s.take_partial();
        }
        #[test]
        fn short_lines_reassemble_exactly(chunks in proptest::collection::vec(proptest::collection::vec(prop_oneof![Just(b'\n'), any::<u8>()], 0..64), 0..16)) {
            let input = chunks.concat();
            let mut s = Splitter::new();
            let mut got: Vec<Vec<u8>> = Vec::new();
            for c in &chunks { s.push(c, |l| got.push(l.to_vec())); }
            let mut want: Vec<Vec<u8>> = input.split(|b| *b == b'\n').map(<[u8]>::to_vec).collect();
            let last = want.pop().unwrap();
            prop_assert_eq!(got, want);
            prop_assert_eq!(s.take_partial(), if last.is_empty() { None } else { Some(last) });
        }
    }
}
