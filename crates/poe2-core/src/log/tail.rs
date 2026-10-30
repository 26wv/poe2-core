//! `LogTail`, the poll-based follower of a client log (docs/research/client-log.md "File properties", Unknown 4 and 6; ADRs 0010, 0014).

use std::collections::VecDeque;
use std::fs::File;
use std::io::{self, ErrorKind, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use super::LogLine;
use super::split::{CHUNK, Splitter, decode};

/// A follower of one client log path that yields the log lines appended to it.
///
/// Each poll opens the path again, so a file deleted and created anew is followed when it is
/// shorter than the read position; one at least as long is read from the old position (ADR 0010).
/// [`LogTail::next_line`] never blocks; the caller picks the poll interval (ADR 0014).
pub struct LogTail {
    path: PathBuf,
    pos: u64,
    splitter: Splitter,
    queue: VecDeque<LogLine>,
    file: Option<File>,
    buf: Vec<u8>,
}

impl std::fmt::Debug for LogTail {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LogTail")
            .field("path", &self.path)
            .field("pos", &self.pos)
            .finish_non_exhaustive()
    }
}

impl LogTail {
    /// Follows `path` from its current end; a partial last line already there is skipped.
    /// Fails if the path can't be opened now or is not a regular file.
    pub fn open(path: impl Into<PathBuf>) -> io::Result<LogTail> {
        let path = path.into();
        let mut file = open_file(&path)?;
        let mut tail = LogTail::new(path);
        tail.pos = file.metadata()?.len();
        if let Some(last) = tail.pos.checked_sub(1) {
            let mut byte = [0];
            file.seek(SeekFrom::Start(last))?;
            file.read_exact(&mut byte)?;
            if byte != *b"\n" {
                tail.splitter.skip_to_next_line();
            }
        }
        Ok(tail)
    }

    /// Follows `path` from byte 0. Fails if the path can't be opened now or is not a regular file.
    pub fn open_from_start(path: impl Into<PathBuf>) -> io::Result<LogTail> {
        let path = path.into();
        open_file(&path)?;
        Ok(LogTail::new(path))
    }

    fn new(path: PathBuf) -> LogTail {
        LogTail {
            path,
            pos: 0,
            splitter: Splitter::new(),
            queue: VecDeque::new(),
            file: None,
            buf: vec![0; CHUNK],
        }
    }

    /// Returns the next complete line available now, or `Ok(None)` once caught up; never sleeps.
    /// A missing file counts as no new data; any other IO error, or a path that is no longer a
    /// regular file, is returned.
    pub fn next_line(&mut self) -> io::Result<Option<LogLine>> {
        loop {
            if let Some(line) = self.queue.pop_front() {
                return Ok(Some(line));
            }
            let file = match &mut self.file {
                Some(file) => file,
                None => match self.start_poll()? {
                    Some(file) => self.file.insert(file),
                    // A truncation to empty may have queued the partial line.
                    None => return Ok(self.queue.pop_front()),
                },
            };
            match file.read(&mut self.buf) {
                Ok(0) => {
                    self.file = None;
                    return Ok(None);
                }
                Ok(n) => {
                    // `min` guards against a reader that reports more bytes than the buffer holds.
                    let bytes = &self.buf[..n.min(CHUNK)];
                    self.pos = self.pos.saturating_add(bytes.len() as u64);
                    let queue = &mut self.queue;
                    self.splitter.push(bytes, |l| queue.extend(decode(l)));
                }
                Err(e) if e.kind() == ErrorKind::Interrupted => {}
                Err(e) => {
                    self.file = None;
                    return Err(e);
                }
            }
        }
    }

    /// Opens the path at the read position; `None` when it is missing or has no new bytes.
    /// A file shorter than the read position is a truncation: the partial line is queued and
    /// reading restarts at 0 with a fresh splitter (ADR 0010 "Shrinking").
    fn start_poll(&mut self) -> io::Result<Option<File>> {
        let mut file = match open_file(&self.path) {
            Ok(file) => file,
            Err(e) if e.kind() == ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(e),
        };
        let len = file.metadata()?.len();
        // Known limit: a recreated file at least `pos` long is read from `pos` (ADR 0010 gap); add a file-identity check if tools recreate it large.
        if len < self.pos {
            self.queue
                .extend(self.splitter.take_partial().as_deref().and_then(decode));
            // A fresh splitter also clears a long line's skip, which would eat the new first line.
            self.splitter = Splitter::new();
            self.pos = 0;
        }
        if len <= self.pos {
            return Ok(None);
        }
        file.seek(SeekFrom::Start(self.pos))?;
        Ok(Some(file))
    }
}

/// Opens `path` if it is a regular file (links followed). A FIFO or device would block or act on
/// open, breaking the never-blocks promise (ADR 0014), so it fails with `InvalidInput`.
// Known limit: the path can be swapped between the check and the open; open with O_NONBLOCK if that matters.
fn open_file(path: &Path) -> io::Result<File> {
    if !path.metadata()?.is_file() {
        return Err(io::Error::new(
            ErrorKind::InvalidInput,
            "not a regular file",
        ));
    }
    File::open(path)
}

#[cfg(test)]
mod tests {
    use super::LogTail;
    use crate::log::Event;
    use crate::test_support::TempDir;
    use std::{
        fs::{self, OpenOptions},
        io::Write,
        path::Path,
    };
    pub(super) fn append(p: &Path, b: &[u8]) {
        OpenOptions::new()
            .create(true)
            .append(true)
            .open(p)
            .unwrap()
            .write_all(b)
            .unwrap();
    }
    pub(super) fn raws(t: &mut LogTail) -> Vec<String> {
        let mut v = Vec::new();
        while let Some(l) = t.next_line().unwrap() {
            v.push(match l.event {
                Event::Unknown { raw } => raw,
                e => format!("{e:?}"),
            });
        }
        v
    }
    #[test]
    fn from_start_reads_existing_then_appended_lines() {
        let t = TempDir::new();
        let p = t.join("Client.txt");
        append(&p, b"a\nb\n");
        let mut tail = LogTail::open_from_start(&p).unwrap();
        assert_eq!(raws(&mut tail), ["a", "b"]);
        append(&p, b"c\n");
        assert_eq!(raws(&mut tail), ["c"]);
        assert!(raws(&mut tail).is_empty());
    }
    #[test]
    fn from_end_skips_existing_lines_and_a_cut_line() {
        let t = TempDir::new();
        let p = t.join("Client.txt");
        append(&p, b"old1\nold2\n");
        let mut tail = LogTail::open(&p).unwrap();
        assert!(raws(&mut tail).is_empty());
        append(&p, b"new\n");
        assert_eq!(raws(&mut tail), ["new"]);
        let q = t.join("mid.txt");
        append(&q, b"old\npart");
        let mut mid = LogTail::open(&q).unwrap();
        append(&q, b"ial\nnext\n");
        assert_eq!(raws(&mut mid), ["next"]);
    }
    #[test]
    fn holds_a_partial_last_line_until_its_newline() {
        let t = TempDir::new();
        let p = t.join("Client.txt");
        append(&p, b"a\nb");
        let mut tail = LogTail::open_from_start(&p).unwrap();
        assert_eq!(raws(&mut tail), ["a"]);
        append(&p, b"c\n");
        assert_eq!(raws(&mut tail), ["bc"]);
    }
    #[test]
    fn crlf_bom_invalid_utf8_and_blank_lines() {
        let t = TempDir::new();
        let p = t.join("Client.txt");
        append(
            &p,
            b"\xef\xbb\xbf2026/01/02 03:04:05 1 a [INFO Client 1] x\r\n\xff\xfe\r\n\r\n",
        );
        let mut tail = LogTail::open_from_start(&p).unwrap();
        let first = tail.next_line().unwrap().unwrap();
        assert!(first.prefix.is_some());
        assert_eq!(raws(&mut tail), ["\u{fffd}\u{fffd}"]);
    }
    #[test]
    fn opening_a_missing_file_fails_and_an_empty_file_works() {
        let t = TempDir::new();
        assert_eq!(
            LogTail::open(t.join("none")).unwrap_err().kind(),
            std::io::ErrorKind::NotFound
        );
        assert_eq!(
            LogTail::open_from_start(t.join("none")).unwrap_err().kind(),
            std::io::ErrorKind::NotFound
        );
        let p = t.join("empty.txt");
        fs::write(&p, "").unwrap();
        let mut tail = LogTail::open(&p).unwrap();
        append(&p, b"x\n");
        assert_eq!(raws(&mut tail), ["x"]);
    }
    #[cfg(unix)]
    #[test]
    fn a_fifo_is_refused_without_blocking() {
        use crate::test_support::{mkfifo, within_5s};
        let t = TempDir::new();
        let fifo = t.join("fifo");
        mkfifo(&fifo);
        let not_a_file = |e: std::io::Error| {
            assert_eq!(e.kind(), std::io::ErrorKind::InvalidInput);
            assert_eq!(e.to_string(), "not a regular file");
        };
        let f = fifo.clone();
        not_a_file(within_5s(move || LogTail::open(f)).unwrap_err());
        let f = fifo.clone();
        not_a_file(within_5s(move || LogTail::open_from_start(f)).unwrap_err());
        // A log replaced by a FIFO after opening fails the next poll.
        let p = t.join("Client.txt");
        append(&p, b"a\n");
        let mut tail = LogTail::open(&p).unwrap();
        fs::remove_file(&p).unwrap();
        mkfifo(&p);
        not_a_file(within_5s(move || tail.next_line()).unwrap_err());
    }
    #[test]
    fn truncation_flushes_the_partial_line_and_restarts_at_zero() {
        let t = TempDir::new();
        let p = t.join("Client.txt");
        append(&p, b"a\nb\npartial"); // 11 bytes; the read position ends at 11
        let mut tail = LogTail::open_from_start(&p).unwrap();
        assert_eq!(raws(&mut tail), ["a", "b"]);
        fs::write(&p, "\u{feff}new\r\n").unwrap(); // a backup tool recreates it smaller (8 bytes), with a BOM
        assert_eq!(raws(&mut tail), ["partial", "new"]);
    }
    #[test]
    fn truncation_to_empty_yields_the_partial_line_on_that_poll() {
        let t = TempDir::new();
        let p = t.join("Client.txt");
        append(&p, b"a\npartial");
        let mut tail = LogTail::open_from_start(&p).unwrap();
        assert_eq!(raws(&mut tail), ["a"]);
        fs::write(&p, "").unwrap();
        assert_eq!(raws(&mut tail), ["partial"]);
        append(&p, b"b\n");
        assert_eq!(raws(&mut tail), ["b"]);
    }
    #[test]
    fn truncation_during_a_long_line_skip_keeps_the_new_first_line() {
        let t = TempDir::new();
        let p = t.join("Client.txt");
        append(&p, &vec![b'x'; crate::log::split::MAX_LINE_BYTES + 10]);
        let mut tail = LogTail::open_from_start(&p).unwrap();
        assert_eq!(raws(&mut tail).len(), 1);
        fs::write(&p, "first\n").unwrap();
        assert_eq!(raws(&mut tail), ["first"]);
    }
    #[test]
    fn rotation_by_delete_and_recreate() {
        let t = TempDir::new();
        let p = t.join("Client.txt");
        append(&p, b"one\ntwo\n");
        let mut tail = LogTail::open_from_start(&p).unwrap();
        assert_eq!(raws(&mut tail).len(), 2);
        fs::remove_file(&p).unwrap();
        assert!(raws(&mut tail).is_empty());
        fs::write(&p, "fresh\n").unwrap();
        assert_eq!(raws(&mut tail), ["fresh"]);
    }
    #[test]
    fn absurd_lines_are_cut_at_the_cap() {
        let t = TempDir::new();
        let p = t.join("Client.txt");
        fs::write(&p, "").unwrap();
        let mut tail = LogTail::open_from_start(&p).unwrap();
        append(&p, &vec![b'x'; 4 << 20]);
        append(&p, b"\nok\n");
        let got = raws(&mut tail);
        assert_eq!(
            (got.len(), got[0].len(), got[1].as_str()),
            (2, crate::log::split::MAX_LINE_BYTES, "ok")
        );
    }
    mod props {
        use super::*;
        use proptest::prelude::*;
        #[derive(Debug, Clone)]
        enum Op {
            Append(Vec<u8>),
            Truncate(u64),
            Delete,
            Poll,
        }
        proptest! {
            #![proptest_config(ProptestConfig::with_cases(32))]
            #[test]
            fn file_operations_never_panic(ops in proptest::collection::vec(prop_oneof![
                proptest::collection::vec(any::<u8>(), 0..200).prop_map(Op::Append), (0u64..400).prop_map(Op::Truncate), Just(Op::Delete), Just(Op::Poll)], 0..24)) {
                let t = TempDir::new();
                let p = t.join("Client.txt");
                fs::write(&p, "").unwrap();
                let mut tail = LogTail::open_from_start(&p).unwrap();
                for op in ops {
                    match op {
                        Op::Append(b) => append(&p, &b),
                        Op::Truncate(n) => { if let Ok(f) = OpenOptions::new().write(true).open(&p) { f.set_len(n).unwrap(); } }
                        Op::Delete => { let _ = fs::remove_file(&p); }
                        Op::Poll => while let Some(l) = tail.next_line().unwrap() {
                            if let Event::Unknown { raw } = l.event { prop_assert!(raw.len() <= 3 * crate::log::split::MAX_LINE_BYTES); }
                        },
                    }
                }
            }
        }
    }
}
