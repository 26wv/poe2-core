# ADR 0002: All platform probing goes through the `System` trait

Status: accepted, 2026-10-23

**Context.** The locator covers Windows (registry, known folders, Epic manifests) and Linux (Steam roots, Wine prefixes), and Windows logic must be testable on Linux CI and vice versa. Real probing reads the user's disk, registry and environment, which tests can't control. The research adds two needs. Ranking needs modified times. De-duplication needs canonical paths, because `~/.steam/steam` and `~/.steam/root` are links to the same Steam root, and umu and Bottles link one prefix user to another ([linux-paths.md](../research/linux-paths.md) Q1, Q8).

**Decision.**

1. In `locate`, every read of the environment, the registry or the filesystem goes through one trait, `System`. Locator code never calls `std::env`, `std::fs` or Windows APIs directly. `System` provides the following (the implementation chooses the exact signatures):
   - environment variables (`POE2_INSTALL_DIR`, `POE2_USER_DIR`, `XDG_DATA_HOME`, `XDG_CONFIG_HOME`, `WINEPREFIX`, `PROGRAMDATA`, `USERPROFILE`, …);
   - the home folder;
   - the Windows Documents known folder (unverified, so that a dangling OneDrive redirection still comes back; ADR 0006);
   - registry string reads: hive (HKCU or HKLM) + subkey path + value name. `WOW6432Node` is part of the subkey path, not a flag;
   - files: whether a path exists, reading a file (size-capped), the entry names of a directory (sorted, so discovery order is deterministic), a path's modified time, and its canonical path;
   - its **target OS**, Windows or Linux.
2. The real system is cfg-gated per host. On Windows the registry and known-folder reads are real; on other hosts they return nothing. It reports the host as its target OS, and any non-Windows host counts as Linux.
3. `FakeSystem` is an in-memory `System` built for a chosen target OS. It holds files with contents and modified times, directories, links, registry values and environment variables. In v1 it is test-only and crate-private; making it public later is an additive change.
4. `discover_with(sys)` always runs the env-override search, then the searches for `sys`'s target OS:
   - Windows: Steam (registry → library lists → app manifests), standalone, Epic, and the Documents known folder;
   - Linux: Steam (native, Flatpak and Snap roots), Proton prefixes, Heroic, Lutris, Bottles and plain Wine.

   Search code contains no `cfg`; only the real system does. `discover()` is `discover_with(&<real system>)`. As a result, the Windows searches run on Linux CI under a Windows `FakeSystem`, and the Linux searches run on Windows CI under a Linux `FakeSystem`.
5. **Target paths are built as text.** A helper joins segments with the target OS's separator (`\` or `/`) and turns `/` into `\` in Windows input (the registry's `SteamPath` uses `/`). The result is stored as a `PathBuf`. On a real host of the same OS that is an ordinary path. On the other host it is an opaque string that only `FakeSystem` interprets. Search code therefore never calls `Path::join`, `parent`, `file_name` or `components` on a target path, and it joins directory entry names through the helper.
6. **Identity.** Windows paths are compared case-insensitively with `\` separators. Linux candidates that exist are reported and compared by their canonical path; the others are compared as written.
7. **Ranking.** Each Discovery list is ordered by these keys: env override first; then candidates that exist; then, for `installs` and `client_log`, the newest `logs/Client.txt` modified time; then discovery order.

**Considered options.**
- `cfg`-gated searches: the simplest option, but Windows logic would never run on Linux CI.
- Running every search on every OS: Linux would probe `C:\…` as a relative path, and `poe2 doctor` would list Windows checks on Linux.
- The `typed-path` crate: correct on every host, but a dependency for what a short helper does.
- Passing the target OS to `discover_with` as an argument: `FakeSystem` needs the target OS anyway, to know how its paths compare.

**Consequences.**
- The real Windows system is the only code that touches Windows APIs (ADR 0006). Windows CI is the first place it compiles. Keep it small.
- Windows searches de-duplicate by comparing paths and never canonicalise, so `\\?\` verbatim paths from `std::fs::canonicalize` never reach the user.
- Reads are size-capped, so a huge or hostile launcher file can't exhaust memory. The implementation picks the cap (about 1 MiB is plenty). A file over the cap counts as unreadable, with a note.
- `LogTail` reads real files directly (ADR 0010). `System` is the locator's seam only.
