# ADR 0006: Use windows-sys for the real Windows system

Status: accepted, 2026-10-23

**Context.** The real Windows system needs two OS calls: the Documents known folder and registry string reads ([windows-paths.md](../research/windows-paths.md) Q5, Q7).
- Documents must come from `SHGetKnownFolderPath(FOLDERID_Documents)`, which follows OneDrive Known Folder Move. Microsoft says not to read `User Shell Folders`, and `%USERPROFILE%\Documents` is only a fallback.
- With `KF_FLAG_DONT_VERIFY` the call returns a redirected path even when its target is gone. That is the OneDrive failure players report, and `poe2 doctor` should show it with `exists: false`. `dirs` fixes the flags at 0 and `known-folders` at `KF_FLAG_DEFAULT`, so both fail instead.

**Decision.**
- Add one dependency, `[target.'cfg(windows)'.dependencies] windows-sys = "0.61"`, with the features `Win32_Foundation`, `Win32_UI_Shell`, `Win32_System_Com` and `Win32_System_Registry`.
- Write two small `unsafe` wrappers in the real Windows system:
  - the Documents known folder, read with `KF_FLAG_DONT_VERIFY` and released with `CoTaskMemFree` whether the call succeeds or fails;
  - a registry string read, with `RegGetValueW` and `RRF_RT_REG_SZ`.
- The crate denies `unsafe_code` everywhere else.
- `%USERPROFILE%\Documents` is listed as a `Source::Fallback` candidate.

**Considered options.**
- `windows-registry` 0.6: a safe registry API that adds 3 small crates. It is acceptable if hand-written registry FFI is rejected. Version 0.100 is not: it needs MSRV 1.95 and duplicates `windows-link`.
- `winreg`: no advantage here.
- `dirs` and `known-folders`: their fixed flags hide the OneDrive case, and `dirs` also brings XDG logic we don't need.
- `steamlocate`: it reads the real machine, bypassing `System`.

**Consequences.**
- No extra crates on Linux, and an MSRV of 1.71, below the 1.85 that edition 2024 already needs.
- Windows CI is the first place this module is compiled. Keep it under about 60 lines.
- `windows-sys` 0.100 will rename its features. Stay on 0.61 until a reason to move appears.
