# Test the Win32 reads on the Windows runner

**Context.** Found in the v0.1.0 code review. The only unsafe code (crates/poe2-core/src/locate/real/win32.rs, used by real.rs:34 and :41) has no test that checks its results on Windows: `target_os_and_platform_reads` asserts only on non-Windows, and the Windows CLI tests check only the JSON shape. win32.rs:37 does `raw.add(i)` and a deref under one SAFETY comment. A registry value that grows between the size query and the read gives `ERROR_MORE_DATA` and so `None`, with no `Known limit:` comment.

**Problem.** A wrong buffer length, a stray trailing NUL or a type-filter mistake would ship silently and break every Windows candidate.

**Acceptance criteria.**
- A `#[cfg(windows)]` test: `ProductName` under `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion` is `Some`, non-empty, with no `\0`; the REG_DWORD `CurrentMajorVersionNumber` and a missing value give `None`; `documents_folder()` is `Some(p)` with `tpath::absolute(Windows, &p).is_some()`.
- The two-op unsafe block is split, each with its own SAFETY comment.
- `ERROR_MORE_DATA` is retried once, or carries a `Known limit:` comment.

**Rank.** should
