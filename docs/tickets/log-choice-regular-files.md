# Prefer regular files when choosing the client log

**Context.** Found in the v0.1.0 code review. Ranking (crates/poe2-core/src/locate/candidates.rs:90-95) orders logs by source, `exists` and mtime only, and `poe2 tail` with no `--log` takes the first existing one (crates/poe2-cli/src/lib.rs:128). A FIFO or a folder named `Client.txt` counts as existing.

**Problem.** A non-regular `Client.txt` with the newest mtime wins, so `poe2 tail` refuses it and exits 2 instead of trying the next log. Non-regular files are refused rather than opened, so nothing hangs, but anyone who can write one candidate folder can still block auto-selection. Related to the mtime trust in docs/tickets/security-md-scope.md.

**Acceptance criteria.**
- Log ranking puts non-regular files after regular ones (or `tail` skips them when choosing); an explicit `--log` is unchanged.
- A test with a FIFO or folder `Client.txt` that has the newest mtime shows the regular log chosen.
- No public API change (ADR 0022), or an ADR records it.

**Rank.** should
