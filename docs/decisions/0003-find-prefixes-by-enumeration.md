# ADR 0003: Find Wine prefixes by enumeration, not by ID

Status: accepted, 2026-10-23

**Context.** Guides disagree on PoE2's compatdata folder. Some use `2694490`, the Steam store app ID; others use `2315204395`, the ID in GGG's own SteamOS example. The second is a shortcut ID (bit 31 set): it differs per user and per shortcut, and it can't be hard-coded ([windows-paths.md](../research/windows-paths.md) Q1, [linux-paths.md](../research/linux-paths.md) Q4). The only list of shortcut IDs is `userdata/<steamid3>/config/shortcuts.vdf`. That file is binary VDF, in a folder named after the Steam account ID. A Proton prefix can sit in any library, not only the game's (linux-paths.md Q3). Heroic sideload app names are random, and PoE2's Epic `AppName` has a single source. The two copies of the library list are documented inconsistently: the Windows research reads both, the Linux research reads one.

**Decision.** The locator recognises PoE2 by its layout on disk, not by launcher IDs.
- **Steam libraries.** These are the Steam root plus every entry in both `steamapps/libraryfolders.vdf` and `config/libraryfolders.vdf`, merged and de-duplicated, on both OSes. In each library, list `steamapps/compatdata/*` and probe every `pfx/` as a prefix. This finds the store prefix (`2694490`) and shortcut prefixes alike. The folder name goes in `note`, marked "store app" or "shortcut".
- **Launcher prefixes.** Take every prefix a launcher records, plus its documented default locations, without filtering by game name or ID:
  - Heroic: the `winePrefix` values in `GamesConfig/*.json` and `config.json`;
  - Lutris: the top-level `game.prefix` in every `games/*.yml`;
  - Bottles: every bottle;
  - plain Wine: `$WINEPREFIX`, `~/.wine` and `~/Games/umu/*`.
- **Probing a prefix.** Check these paths, and also the same paths under `pfx/` for Proton-style prefixes:
  - user dir: `drive_c/users/*/Documents/My Games/Path of Exile 2` (also under the legacy `My Documents`, and skipping `Public`);
  - install: `drive_c/Program Files (x86)/Grinding Gear Games/Path of Exile 2` and the same under `Program Files`.

  Matches are canonicalised and de-duplicated, because umu and Bottles link `users/<unix user>` to `steamuser`.
- **What gets reported.** Enumerated prefixes produce candidates only where PoE2's layout exists. Fixed locations are always listed with `exists`: each Steam root, `compatdata/2694490` in each library that holds PoE2's app manifest, and each launcher's default PoE2 prefix (for example `~/Games/path-of-exile-2` for Lutris).
- **What is never read.** `userdata/`, because it is named after the Steam account ID. The locator also never hard-codes a shortcut ID.
- **No direct probe of the host's `~/Documents/My Games/Path of Exile 2`.** Only a plain-Wine prefix links there, and canonicalising that prefix's candidate already arrives at it.

**Consequences.**
- The cost is one directory listing per library plus a few existence checks per prefix, and there is no binary VDF parser.
- `poe2 doctor` stays short even with hundreds of Proton games. Bug reports made from `poe2 paths --json` don't reveal the user's other games.
- A second PoE2 copy in any prefix is reported too, and ranking decides the order.
- If a user overrides `STEAM_COMPAT_DATA_PATH` in launch options, that is visible only under `userdata/`. Such a prefix is found only through `POE2_USER_DIR`.
