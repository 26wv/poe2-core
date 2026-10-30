# CLI test gaps and read-error flush

**Context.** Found in the v0.1.0 code review.
- crates/poe2-cli/src/lib.rs:105, :155 print the read error before `BufWriter` flushes earlier lines, so on a terminal the error appears above them.
- `poe2 tail` exit-code tests (tests/tail.rs:99-119) use `.output()` with no timeout.
- Untested: `--help` with empty stderr; non-UTF-8 args; the `unwrap_or(2)` fallback (lib.rs:80); a directory as FILE; the `poe2: cannot read` prefix (parse.rs:30 and the tail missing-log test); tail's broken-pipe exit 0; truncation, partial and long lines through the CLI; human-format `parse` end to end; unix `paths --json` exit 0 (`a_synthetic_steam_install_is_printed`, tests/paths.rs:71-83, checks only the plain output); a byte compare of `doctor --json` against `paths --json`.
- tests/doctor.rs:19 `contains("0 of ")` also matches "10 of".
- parse_memory.rs leaves a ~29 MB temp file if the run fails.

**Problem.** Error ordering is confusing, a follow regression hangs instead of failing, and several exit paths are checked only by hand.

**Acceptance criteria.**
- `out.flush()` runs before each `cannot read` message.
- The tail exit-code tests use the `Running` guard with `recv_timeout`.
- Tests cover the cases above; the doctor check matches a line starting with "0 of ".
- The parse_memory temp file is removed on failure (a drop guard).

**Rank.** nice
