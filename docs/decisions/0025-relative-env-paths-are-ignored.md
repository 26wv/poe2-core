# ADR 0025: Relative env paths are ignored

Status: accepted, 2026-10-24. Extends ADR 0020's absolute-only rule to environment variables.

**Context.** ADR 0020 and the Linux searches skip a relative path from launcher data instead of resolving it against the working directory, and the `XDG_*` reads already ignore a value that doesn't start with `/`. The other env values the locator reads as paths were used as given, so a relative `WINEPREFIX`, `POE2_INSTALL_DIR`, `POE2_USER_DIR`, `ProgramFiles`, `ProgramFiles(x86)`, `PROGRAMDATA` or `USERPROFILE` became a candidate relative to the cwd. Running `poe2 tail` inside someone else's folder could then follow their `Client.txt`. Wine itself refuses a relative `WINEPREFIX`, and Windows sets the others as absolute paths, so a relative value never names the real location.

**Decision.**
- Every env value the locator reads as a path goes through one rule, `System::env_path`: it is normalised and used only if absolute (`/…` on Linux; `X:\…` or `\\…` on Windows, since env values may be UNC per ADR 0024). Anything else counts as unset: the override row is not listed, `WINEPREFIX` is not probed, and the Windows variables fall back to their documented defaults (`C:\Program Files (x86)`, `C:\Program Files`, `C:\ProgramData`; no `USERPROFILE` row).
- The `XDG_*` reads keep their existing `/`-prefix filter, which is the same rule on Linux.
- A dropped override is silent, like any other skipped value.

**Considered options.** Warning about a relative `POE2_*` override instead of dropping it: the Discovery has no channel for warnings, and the CLI's `doctor` already shows that no `EnvOverride` row exists.

**Consequences.** A user who sets `POE2_INSTALL_DIR=.` must give an absolute path instead. `HOME` (read through `std::env::home_dir`) is not covered by this rule.
