# Python discover() reads env with the GIL released

**Context.** Found in the v0.1.0 code review. bindings/python/src/lib.rs:31 runs `py.detach(poe2::locate::discover)`. Discovery calls `getenv`, and std's env lock doesn't cover Python, so another thread's `os.environ[...] = …` (`setenv`) can reallocate `environ` while it is walked.

**Problem.** Safe Python code can trigger a data race (UB, a possible segfault) inside the extension.

**Acceptance criteria.**
- `discover()` keeps the GIL (it takes milliseconds), or snapshots the env vars it needs under the GIL.
- `LogTail` calls, which read no env, still release the GIL.
- The choice is noted in the binding's doc comment.

**Rank.** should
