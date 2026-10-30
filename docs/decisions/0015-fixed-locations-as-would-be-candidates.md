# ADR 0015: Fixed locations are listed as would-be candidates

Status: accepted, 2026-10-23

**Context.** ADR 0003 says fixed locations are always listed with `exists`, and `poe2 doctor` must show every location it checked, even on a machine without PoE2. But a Steam root or a launcher's default prefix is neither an install nor a user dir, and `Discovery` has only its five lists (GLOSSARY). The research also leaves some Sources open: Snap Steam, plain Wine and the Windows default Steam folder have no variant of their own.

**Decision.**
- `Discovery` keeps exactly its five lists. A fixed location appears as the **would-be candidate** it would hold, with a note. Its `exists` is false unless that path exists. The rows are:
  - Each Steam root probed, when none of its libraries holds PoE2's app manifest: the install `<root>/steamapps/common/Path of Exile 2`, noted `Steam root not found` or `no app manifest for 2694490`.
  - On Linux, each library that holds the app manifest, when its store prefix has no user dir yet: the user dir `<library>/steamapps/compatdata/2694490/pfx/drive_c/users/steamuser/Documents/My Games/Path of Exile 2`, noted `compatdata/2694490 (store app); created on first launch`.
  - Each fixed Wine prefix where probing finds nothing: the install `<prefix>/drive_c/Program Files (x86)/Grinding Gear Games/Path of Exile 2`. The fixed prefixes are Lutris's `~/Games/path-of-exile-2`, Heroic's `~/Games/Heroic/Prefixes/Path of Exile 2`, `$WINEPREFIX` and `~/.wine`.
  - On Windows, always: both standalone default folders, the user dir under the Documents known folder, and the user dir under `%USERPROFILE%\Documents`.
- Epic and Bottles have no fixed location in v1, so they appear only when something is found.
- Each note says how its candidate was found, for example `app manifest in library <path>`, `Lutris games/<file>` or `compatdata/<id> (shortcut)`.
  - A launcher file that can't be read adds `; <path>: <error>` to its root's note.
  - A file over the 1 MiB read cap (ADR 0002) fails with `larger than 1 MiB`.
- Sources:
  - Snap Steam roots are `Steam`.
  - `$WINEPREFIX`, `~/.wine` and umu prefixes are `Fallback`.
  - The Windows default Steam folder (`%ProgramFiles(x86)%\Steam`) is `Fallback`. A Steam root read from the registry is `Steam`.
  - Everything found through a Steam root gets that root's Source.

**Considered options.** A sixth `checked` list of launcher folders: `paths --json` would grow a second vocabulary, and the five lists already say where PoE2 would be.

**Consequences.** On a Linux machine without PoE2, `doctor` lists about a dozen rows marked `no`, which is the point. Ranking puts every existing candidate above them.
