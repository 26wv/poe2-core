# SECURITY.md scope and the mtime ranking

**Context.** Found in the v0.1.0 code review. SECURITY.md's scope list (:17-22) and bug list (:27-29) omit env vars, registry values, CLI args and the bindings (:24 names only the library and the CLI). Ranking (crates/poe2-core/src/locate/candidates.rs:90-95) trusts mtimes, so a planted, future-dated `Client.txt` in any candidate is the one `poe2 tail` follows.

**Problem.** Reporters can't tell what is in scope, and the trust placed in mtimes is undocumented.

**Acceptance criteria.**
- Scope lists env vars, registry values, CLI args and both bindings.
- SECURITY.md or the README states the mtime trust and that the `following <path>` stderr line shows which log is followed.
- Optional: an ADR on preferring EnvOverride, then the launcher's own record, over mtime order.

**Rank.** nice
