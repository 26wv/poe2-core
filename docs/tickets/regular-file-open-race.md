# Close the regular-file check-then-open race

**Context.** Found in the v0.1.0 code review. `RealSystem::read` (crates/poe2-core/src/locate/real.rs:53-63) checks `fs::metadata(path)?.is_file()` and then calls `File::open`; its `Known limit:` comment at real.rs:55 records the gap. `LogTail`'s `open_file` (crates/poe2-core/src/log/tail.rs:137-144) does the same, and runs on every poll (tail.rs:112), which `poe2 tail` repeats every 250 ms (crates/poe2-cli/src/lib.rs:154).

**Problem.** Someone who can write the folder can swap the path for a FIFO between the check and the open, and the open then blocks: `poe2 tail` or Node's sync `nextLine` hangs. Because `LogTail` re-checks on every poll, the window recurs, so the attacker gets unlimited attempts.

**Acceptance criteria.**
- On Linux both reads open with `O_NONBLOCK` (`OpenOptionsExt::custom_flags`) and then check `is_file()` on the opened handle (`file.metadata()`), not on the path; a non-file still gives `InvalidInput` "not a regular file". There is no libc dependency, so the flag is a per-target constant with a comment citing its source (or an ADR picks another equivalent).
- One shared helper serves real.rs and tail.rs, and the `Known limit:` comment at real.rs:55 is removed.
- The existing FIFO tests (real.rs:141, tail.rs:244) still pass under `within_5s`; a test opens a FIFO through the new path and gets the error without blocking.

**Rank.** should
