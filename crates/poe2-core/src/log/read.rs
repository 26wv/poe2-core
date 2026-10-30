//! `read_lines` for offline reads (docs/research/client-log.md "File properties", Unknown 4; ADRs 0008, 0014).

use std::collections::VecDeque;
use std::io::{self, ErrorKind, Read};

use super::LogLine;
use super::split::{CHUNK, Splitter, decode};

/// Parses every non-blank line of `reader` in order, including an unterminated last line.
/// Reads 64 KiB at a time; memory stays bounded whatever the input (ADR 0014).
pub fn read_lines<R: Read>(reader: R) -> ReadLines<R> {
    ReadLines {
        reader,
        buf: vec![0; CHUNK],
        splitter: Splitter::new(),
        queued: VecDeque::new(),
        done: false,
    }
}

/// The iterator [`read_lines`] returns: each item is a parsed log line, or the IO error that ended it.
pub struct ReadLines<R> {
    reader: R,
    buf: Vec<u8>,
    splitter: Splitter,
    queued: VecDeque<LogLine>,
    done: bool,
}

impl<R: Read> Iterator for ReadLines<R> {
    type Item = io::Result<LogLine>;

    /// Yields the next line; an IO error is yielded once, then `None`.
    fn next(&mut self) -> Option<io::Result<LogLine>> {
        while self.queued.is_empty() && !self.done {
            match self.reader.read(&mut self.buf) {
                Ok(0) => {
                    self.done = true;
                    self.queued
                        .extend(self.splitter.take_partial().and_then(|l| decode(&l)));
                }
                Ok(n) => {
                    // `min` guards against a reader that reports more bytes than the buffer holds.
                    let bytes = &self.buf[..n.min(CHUNK)];
                    let queued = &mut self.queued;
                    self.splitter.push(bytes, |l| queued.extend(decode(l)));
                }
                Err(e) if e.kind() == ErrorKind::Interrupted => {}
                Err(e) => {
                    self.done = true;
                    return Some(Err(e));
                }
            }
        }
        self.queued.pop_front().map(Ok)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::log::Event;
    use std::io::{self, Cursor, Read};
    fn raws<R: Read>(r: R) -> Vec<String> {
        read_lines(r)
            .map(|l| match l.unwrap().event {
                Event::Unknown { raw } => raw,
                e => format!("{e:?}"),
            })
            .collect()
    }
    #[test]
    fn yields_every_non_blank_line_including_an_unterminated_last_one() {
        assert_eq!(
            raws(Cursor::new(b"a\r\n\r\nb\n\xffc".to_vec())),
            ["a", "b", "\u{fffd}c"]
        );
    }
    #[test]
    fn a_bom_does_not_hide_the_prefix() {
        let line = read_lines(Cursor::new(
            "\u{feff}2026/01/02 03:04:05 1 a [INFO Client 1] x\n",
        ))
        .next()
        .unwrap()
        .unwrap();
        assert!(line.prefix.is_some());
    }
    #[test]
    fn long_lines_are_cut_and_tiny_reads_work() {
        struct OneByte(Cursor<Vec<u8>>);
        impl Read for OneByte {
            fn read(&mut self, b: &mut [u8]) -> io::Result<usize> {
                let n = b.len().min(1);
                self.0.read(&mut b[..n])
            }
        }
        let mut input = vec![b'x'; 200 * 1024];
        input.extend_from_slice(b"\nok\n");
        let got = raws(OneByte(Cursor::new(input)));
        assert_eq!(
            (got.len(), got[0].len(), got[1].as_str()),
            (2, crate::log::split::MAX_LINE_BYTES, "ok")
        );
    }
    #[test]
    fn an_io_error_is_yielded_once() {
        struct Fails(bool);
        impl Read for Fails {
            fn read(&mut self, b: &mut [u8]) -> io::Result<usize> {
                if self.0 {
                    Err(io::Error::other("boom"))
                } else {
                    self.0 = true;
                    b[..2].copy_from_slice(b"a\n");
                    Ok(2)
                }
            }
        }
        let items: Vec<_> = read_lines(Fails(false)).collect();
        assert_eq!(items.len(), 2);
        assert!(items[0].is_ok() && items[1].is_err());
    }
}
