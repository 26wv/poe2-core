# Reject Windows look-alike installdir names

**Context.** Found in the v0.1.0 code review. `stays_inside` (crates/poe2-core/src/locate/steam.rs:227-230) rejects `/`, `\`, `:`, `.` and `..`, but not `...`, `.. ` or other names ending in a dot or space, which Win32 trims from the last component, nor control characters or NUL.

**Problem.** `installdir "..."` makes `steamapps\common\...` name `common` itself, so a wrong row is listed as existing. Impact is limited to the same, attacker-written library.

**Acceptance criteria.**
- `stays_inside` also rejects names ending in `.` or a space (`dir.ends_with(['.', ' '])` covers `.` and `..`) and names with control characters or NUL.
- `"..."`, `".. "` and a control-character name join the test list at steam.rs:475-483.

**Rank.** should
