# Verify Linux launcher layouts on real installs

**Context.** Several Linux claims come from launcher source code or are inferred, not observed ([linux-paths.md](../research/linux-paths.md) Unknown, Conflicting):
- the forms Flatpak Steam's library `"0"` string takes, and libraries added under `$HOME` through overrides;
- whether Snap's `libraryfolders.vdf` holds host paths;
- Heroic Flatpak prefixes outside `~/Games/Heroic`;
- Bottles Flatpak paths recorded as `/var/data/…`;
- whether Bottles' YAML folds the way Lutris's does;
- `SteamApps` capitalisation in old libraries (single source).
- Flatpak Lutris (`~/.var/app/net.lutris.Lutris/{config,data}/lutris`, linux-paths.md Q6), which v1 doesn't search at all.

ADR 0004 covers the Flatpak cases by probing both the recorded path and the remapped one.

**Needed.** For Flatpak Steam, Snap Steam, Heroic (native and Flatpak), Lutris (Flatpak) and Bottles (native and Flatpak), anonymised copies of these, plus the real prefix paths:
- `libraryfolders.vdf`;
- `GamesConfig/*.json`;
- `placeholder.yml` and `data.yml`.

**Acceptance criteria.**
- linux-paths.md records the observations.
- `FakeSystem` fixtures use the observed layouts.
- Any remap that proves unnecessary is removed.
- Flatpak Lutris dirs are searched, with a test, if the layout is confirmed.
