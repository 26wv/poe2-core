# ADR 0014: LogTail is a non-blocking pull, and offline reads use `read_lines`

Status: accepted, 2026-10-23

**Context.** `poe2 tail` loops until it is killed, `poe2 parse` reads a file to its end, and each binding needs a tail iterator. Blocking is natural in Python but would freeze Node's event loop. ADR 0010 fixes the behaviour (poll the path, start at the end, handle truncation, partial lines and long lines) and leaves the cap and the API to the implementation. A poll that returned all new lines as one `Vec` would use memory in proportion to the file under `--from-start`.

**Decision.**

```rust
impl LogTail {
    pub fn open(path: impl Into<PathBuf>) -> io::Result<LogTail>;            // start at the end
    pub fn open_from_start(path: impl Into<PathBuf>) -> io::Result<LogTail>; // start at byte 0
    pub fn next_line(&mut self) -> io::Result<Option<LogLine>>;              // never blocks
}
pub fn read_lines<R: Read>(reader: R) -> ReadLines<R>; // Iterator<Item = io::Result<LogLine>>
```

- `next_line` returns the next complete line if one is available now, and `Ok(None)` once it has caught up with the file. It never sleeps, so the caller sets the poll interval:
  - the CLI sleeps 250 ms;
  - the Python iterator blocks, sleeping 250 ms at a time with the GIL released;
  - Node code drains the tail from a timer.
- A poll starts when no decoded line is queued. It opens the path, compares the file's length with the read position, and reads the new bytes in 64 KiB chunks.
  - A missing file counts as no new data, because rotation deletes the file and creates it again. Any other error is returned.
  - `open` and `open_from_start` fail if the path can't be opened at that moment.
- `read_lines` is for offline reads (`poe2 parse`). It yields every non-blank line in order, including an unterminated last line at the end of the input. An IO error is yielded once and ends the iteration.
- Both use one internal line splitter. A line longer than the **line cap** of 64 KiB is cut at the cap, and the rest is skipped up to the next `\n` (ADR 0008, 0010).
- Each reader holds at most one 64 KiB chunk, one held line of up to 64 KiB, and the lines decoded from one chunk.
- `LogTail` doesn't implement `Iterator`: an iterator that ends and later yields again surprises callers.

**Consequences.** One core API serves all three callers, and each language adds only its own way of waiting. In v1 the cap is a constant, not an option.
