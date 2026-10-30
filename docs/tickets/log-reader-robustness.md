# Log reader and tail: shared chunk loop, bounds and test gaps

**Context.** Found in the v0.1.0 code review.
- The read-a-chunk loop (Interrupted retry, `min` guard, `splitter.push`) is copied in log/read.rs:35-54 and log/tail.rs:87-104, and the `raws` test helper in both test modules.
- A 64 KiB chunk of 1-byte lines queues 32,768 `LogLine`s, about 7.9 MB (read.rs:46, tail.rs:97). Bounded per ADR 0014, but undocumented.
- `ReadLines` (read.rs:30) is fused in practice but doesn't implement `FusedIterator`.
- The Interrupted retry is untested. A file that shrinks between `metadata()` and the last-byte read in `LogTail::open` (tail.rs:42-44) gives `UnexpectedEof` instead of starting at the new end.
- The splitter's held buffer can reach ~2x the cap; split.rs:144 uses `assert!` inside a proptest. tail.rs:348's 3x length assert can't fail; bounded_memory.rs:77-93 leaves a 16 MiB temp file on panic.

**Problem.** Near-copies must change together, a memory bound is undocumented, and some tests can't fail.

**Acceptance criteria.**
- One `split.rs` helper reads a chunk into the queue (Interrupted retried inside, with a test); `raws` moves to test_support.rs.
- `read_lines` docs state the per-chunk queue bound (or lines are split lazily).
- `impl FusedIterator for ReadLines<R>`.
- The shrink race starts at the new end, or carries a `Known limit:` comment.
- The held-buffer bound is documented, `prop_assert!` is used, the vacuous assert is fixed or removed, and the temp file is removed by a drop guard.

**Rank.** nice
