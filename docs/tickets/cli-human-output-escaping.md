# Escape control characters and pad by display width in human CLI output

**Context.** Found in the v0.1.0 code review. `doctor`, `paths`, `tail`'s `following …` line and the `cannot read` message (crates/poe2-cli/src/report.rs:24, :46-47; lib.rs:146, :200) print launcher-derived text raw. A Lutris `games/` folder whose name holds an OSC escape put raw ESC bytes into `poe2 doctor`'s note (reproduced in a temp HOME). report.rs:54 and :61 pad by char count, not display width. JSON output is already escaped.

**Problem.** A crafted launcher file can set the terminal title, write the clipboard (OSC 52) or spoof links (CWE-150). CJK names misalign the table.

**Acceptance criteria.**
- Human output escapes `char::is_control` characters (for example as `\u{1b}`); JSON output stays byte-exact.
- A test with an ESC in a folder name checks the escaped `doctor` output.
- Table padding uses display width (a CJK case is tested), or the ticket records why char count stays.

**Rank.** should
