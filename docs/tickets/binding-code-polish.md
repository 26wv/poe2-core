# Binding code polish: error kinds, json.loads, unsafe lint

**Context.** Found in the v0.1.0 code review. Node (bindings/node/src/lib.rs:5-7, :51, :59) repeats `from_reason(e.to_string())` and `serde_json::to_string(..).map_err(js)?`, and its errors carry no `code` (`ENOENT` …). Python's OSError subclasses (bindings/python/src/lib.rs:66, :72) carry no `errno` or `filename`; python/src/lib.rs:16 imports `json` and looks up `loads` on every line. Neither binding has `#![deny(unsafe_code)]`, which the core and CLI crates set. python/src/lib.rs:45 doesn't say a second thread using the same `LogTail` gets "Already mutably borrowed", which `check_signals` (lib.rs:88) can also trigger through a signal handler.

**Problem.** Callers can't branch on the error kind without parsing messages; a hot path does needless lookups; the unsafe rule isn't enforced in the bindings.

**Acceptance criteria.**
- Node: one error helper sets `code` from `e.kind()`; `parse_json` takes a `serde_json::Result<String>` like Python's `loads`.
- Python: OSError is built as `(errno, strerror, filename)` when the OS gives an errno; `json.loads` is cached once (`PyOnceLock`).
- Both bindings build with `#![deny(unsafe_code)]`.
- The `LogTail` doc names the "Already mutably borrowed" case, including signal handlers.

**Rank.** nice
