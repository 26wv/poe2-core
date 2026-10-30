# Binding test gaps

**Context.** Found in the v0.1.0 code review.
- Python's `test_discover` checks only top-level keys; Node also checks candidate field types.
- Node's test deletes a fixed env list, not every `POE2_*`/`XDG_*`, and checks only `installs`.
- No test for `nextLine()` throwing mid-tail (Node), or for `LogTail(p)` starting at the end by default (Python).
- Node test files are listed by hand in package.json; `node --test` with no arguments finds `test/*.test.mjs` on Node 20+.
- bindings/node/test/smoke.test.mjs:10 never removes its `mkdtempSync` HOME.
- On Windows the smoke tests still run `discover()` against the real registry and Documents folder (an empty temp HOME and cleared env vars can't isolate them).

**Problem.** The bindings check less than each other, leak a temp folder per run, and on a contributor's Windows machine a failing assertion could print real paths.

**Acceptance criteria.**
- Python checks candidate field types as Node does; Node checks every Discovery list and clears env by prefix.
- Tests cover the mid-tail throw and the default start-at-end.
- `node --test` discovers test files; the temp HOME is removed in `after()`.
- Windows smoke tests assert only shapes and counts and never echo a Discovery; the limit is noted in an ADR or the test file.

**Rank.** nice
