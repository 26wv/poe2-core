# Reject Windows DOS device names in launcher paths

**Context.** Found in the v0.1.0 code review. `tpath::absolute` (crates/poe2-core/src/locate/tpath.rs:37-47) accepts any `X:\…` path, so launcher data such as `D:\COM1`, `D:\x\LPT1` or a path ending in `NUL` passes. Older Windows maps these names to `\\.\…` devices in any folder, which ADR 0024 meant to keep out.

**Problem.** Probing such a path can open a device instead of a file. Rare, and only the last component matters on current Windows.

**Acceptance criteria.**
- `absolute` rejects a Windows path whose any component, with extension and trailing dots or spaces stripped, is `CON`, `PRN`, `AUX`, `NUL`, `COM1`-`COM9` or `LPT1`-`LPT9` (case-insensitive); or ADR 0024's Consequences records why it is accepted.
- tpath.rs:138's test gains `D:\COM1`, `D:\x\lpt1.txt` and `D:\nul` cases.

**Rank.** nice
