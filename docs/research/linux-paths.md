# Linux paths: how a passive reader finds PoE2's files

Researched 2026-10-23 for the `locate` module. Launcher sources were read at pinned commits (see the links).
Tags: **[1]** = the claim rests on a single source. **[local]** = observed on one Linux test install. Paths are anonymised to `/home/<user>`.
"User dir" means `.../Documents/My Games/Path of Exile 2`, the Windows location in [GGG's developer docs][ggg-dev].

## Confirmed

### Summary

| Launcher / root | Path template (host view) | How to read it | Sources |
|---|---|---|---|
| Steam, native (Valve or distro package) | `~/.steam/steam` and `~/.steam/root` → `${XDG_DATA_HOME:-~/.local/share}/Steam`; legacy `~/Steam` | Canonicalise each candidate, keep the ones with `steamapps/` | [launcher], [local] |
| Steam, Debian/Ubuntu `steam-installer` | `~/.steam/debian-installation`; very old installs use `~/.steam` itself | Same; `~/.steam/steam` and `root` link to it | [deb-steam], [deb-bug] |
| Steam, Flatpak | `~/.var/app/com.valvesoftware.Steam/.local/share/Steam` (old `…/data/Steam`, now a symlink to it) | Probe directly. Never resolve `…/com.valvesoftware.Steam/.steam/*` on the host. Remap library path `$HOME/.local/share/Steam` → this root | [fh-yml], [fh-wrap], [flatpak-perm], [pt-libs] |
| Steam, Snap | `~/snap/steam/common/.local/share/Steam`; with the hidden-folder flag `~/.snap/data/steam/common/.local/share/Steam` | Probe directly | [snap-yml], [snap-launch], [pt-dirs], [snap-hidden] |
| Steam libraries | `<root>/steamapps/libraryfolders.vdf` → `"libraryfolders" { "0" { "path" "…" } "1" {…} }` | KeyValues text; also add the root itself; compare keys case-insensitively | [local], [pt-libfmt] |
| PoE2 Steam install | `<lib>/steamapps/appmanifest_2694490.acf` → `<lib>/steamapps/common/<installdir>` (`Path of Exile 2`) | KeyValues `AppState.installdir` | [local], [pt-acf] |
| Proton prefix (store app) | `<lib>/steamapps/compatdata/2694490/pfx/drive_c/users/steamuser/Documents/My Games/Path of Exile 2` | Check every library; it exists only after the first launch | [proton-main], [s4l-12030], [pt-prefix], [local] |
| Non-Steam shortcut prefixes | `<lib>/steamapps/compatdata/<shortcut-id>/pfx/…` (GGG's docs use `2315204395`) | Glob `compatdata/*/pfx/drive_c/users/*/Documents/My Games/Path of Exile 2`; don't read `shortcuts.vdf` | [ggg-dev], [pt-shortcut], [s4l-10184] |
| Heroic | `${XDG_CONFIG_HOME:-~/.config}/heroic/` or `~/.var/app/com.heroicgameslauncher.hgl/config/heroic/`; `GamesConfig/<appName>.json` → `"<appName>": { "winePrefix": … }` | JSON line scan; probe `<prefix>/drive_c` and `<prefix>/pfx/drive_c` | [h-paths], [h-gc], [h-env], [electron] |
| Heroic defaults | `~/Games/Heroic/Prefixes/<Title>`; upgraded installs use `~/Games/Heroic/Prefixes/default/<Title>`; shared prefix `…/Prefixes/shared` (older: `…/Prefixes/default`) | Fallback glob only | [h-ws], [h-5336] |
| Heroic sideload | `<heroic>/sideload_apps/library.json` → `games[]` with `app_name`, `title`, `install.executable`, `folder_name` | JSON line scan; join on `app_name` | [h-sl-store], [h-sl-lib], [es-cwd] |
| Heroic Epic (legendary) | `<heroic>/legendaryConfig/legendary/installed.json` → `{ "<app_name>": { "install_path", "title", "executable", … } }`; default base `~/Games/Heroic` | JSON line scan | [h-leg], [h-legtype] |
| Lutris | `~/.config/lutris/games/*.yml` if `~/.config/lutris` exists, else `~/.local/share/lutris/games/*.yml`; key `game.prefix` | Scoped YAML scalar read; `pga.db` not needed | [l-settings], [l-7af680d], [local] |
| Lutris PoE2 installers | prefix `$GAMEDIR` = `~/Games/path-of-exile-2`; exe `$GAMEDIR/drive_c/Program Files (x86)/Grinding Gear Games/Path of Exile 2/PathOfExile.exe` | Fallback probe | [l-api], [l-target], [l-gamepath] |
| Bottles | `${XDG_DATA_HOME:-~/.local/share}/bottles/bottles/<name>/` or `~/.var/app/com.usebottles.bottles/data/bottles/bottles/<name>/`; the bottle dir is the prefix | List dirs; follow `placeholder.yml` `Path:`; `data.yml` `custom_bottles_path` | [b-paths], [b-check], [b-data] |
| Plain Wine / umu | `${WINEPREFIX:-~/.wine}`; umu without WINEPREFIX: `~/Games/umu/<GAMEID>` | Fallback probe | [wine-cfg], [umu-default] |
| Install inside any prefix | `<prefix>/drive_c/Program Files (x86)/Grinding Gear Games/Path of Exile 2` | Probe | [l-api] (community) |

### Q1: Steam roots and de-duplication

- Valve's launcher (`bin_steam.sh`, steam-launcher 1.0.0.87, L133–L156 and L185 in [launcher]) creates `~/.steam`, sets `DEFAULTSTEAMDIR="${XDG_DATA_HOME:-$HOME/.local/share}/Steam"`, keeps `CLASSICSTEAMDIR="$HOME/Steam"` as a legacy repair target and runs `ln -fns "$STEAMDIR" ~/.steam/steam`. It honours `XDG_DATA_HOME`, so the root is not always `~/.local/share/Steam`.
- The bootstrap `steam.sh` in the same tarball (L28–L33, L690–L740) sets `STEAMDATA="$STEAMROOT"`. It defines `~/.steam/root` as "the Steam install path for the currently running Steam" and `~/.steam/steam` as "the Steam content path", and links both to that one directory. `~/.steam/steam` is skipped in developer setups (`steam_dev.cfg`). Debian's script says the two can differ "when testing new Steam client binaries" ([deb-steam]). For `steamapps`, prefer `~/.steam/steam`, then `~/.steam/root`.
- Debian/Ubuntu `steam-installer` 1.0.0.87~ds-3, `/usr/games/steam` L50–L73 ([deb-steam]): it follows an existing `~/.steam/steam` or `~/.steam/root` link. If `~/.steam/steam` is a real directory (historic Debian layout), it uses `STEAMDIR=~/.steam`. New installs use `~/.steam/debian-installation`. `README.Debian` says to find the install through the `~/.steam/root` link, which is "usually either ~/.steam/debian-installation or ~/.local/share/Steam". In the historic layout, `~/.steam/root` points at `~/.steam` and `~/.steam/steam` is a real directory ([deb-bug]). Probing `~/.steam/steam` and `~/.steam/root` covers it.
- Flatpak `com.valvesoftware.Steam` has no home access. It gets `--persist=.` plus `/mnt`, `/media` and `/run/media` ([fh-yml] L42–L50). `--persist=.foo` bind-mounts `~/.foo` inside the sandbox onto `~/.var/app/$FLATPAK_ID/.foo` on the host, and `--persist=.` covers all of `~`. `/var/data` inside the sandbox is the app's `~/.var/app/<id>/data` ([flatpak-perm]). Flatpak's default `XDG_DATA_HOME` is `~/.var/app/<id>/data` ([flatpak-xdg]). The wrapper moves that directory to `~/.var/app/com.valvesoftware.Steam/.local/share`, renaming `Steam`, and replaces `data` with a symlink. It then exports `XDG_DATA_HOME=<prefix>/.local/share` with prefix `FLATPAK_STEAM_XDG_DIRS_PREFIX=~/.var/app/com.valvesoftware.Steam` ([fh-yml] L68; [fh-wrap] L220–L295, L330–L346).
- Flatpak caveat: Steam records in-sandbox paths. protontricks rewrites a library path of `~/.local/share/Steam` to `~/.var/app/com.valvesoftware.Steam/.local/share/Steam`, because "~/.local/share/Steam inside a Steam Flatpak sandbox corresponds to a different location" ([pt-libs]). The wrapper reads the current prefix from the `~/.steam/root` link target, which is an in-sandbox absolute path ([fh-wrap] L363–L373). So the per-app `.steam/*` links can point at the native install when resolved on the host. Libraries under `/mnt`, `/media` and `/run/media` have real paths.
- Snap: the `steam` app runs with `HOME: $SNAP_USER_COMMON` ([snap-yml] L548–L551), and `desktop-launch` exports `XDG_DATA_HOME="$SNAP_USER_COMMON/.local/share"` ([snap-launch]). With the launcher above, the root is `~/snap/steam/common/.local/share/Steam`. The snap docs use `~/snap/steam/common/.steam/steam/steamapps/...` ([snap-howto]). protontricks also probes `~/.snap/data/steam/common/.local/share/Steam` ([pt-dirs]), where snapd's `experimental.hidden-snap-folder` moves `SNAP_USER_COMMON` ([snap-hidden]).
- De-duplication: run `fs::canonicalize` on every existing candidate root, and on each library's `steamapps` directory, then dedupe. protontricks resolves and deduplicates its library list the same way ([pt-dedupe]), and its root scan uses an ordered set of resolved paths ([pt-find]). Canonicalise only host-meaningful paths: never the Flatpak's internal `.steam/*` links, and never raw Flatpak library strings before remapping. Non-existent candidates can't be canonicalised, so dedupe those lexically.
- Cross-check: the Rust crate `steamlocate` 2.1.1 (MIT) probes the same roots ([steamlocate]). Unlike this note, it also probes the Flatpak's `.steam/{steam,root}` links, dedupes with a one-level `read_link` instead of `canonicalize`, and doesn't remap Flatpak library paths.
- [local] `~/.steam/steam` and `~/.steam/root` are both symlinks to `/home/<user>/.local/share/Steam`. `bin`, `bin32`, `bin64`, `sdk32` and `sdk64` are links too. There is no `debian-installation`.

### Q2: libraryfolders.vdf and appmanifest on Linux

- It is the same KeyValues text format as on Windows. The current layout ([local], and the dict branch in [pt-libfmt]) is `"libraryfolders" { "0" { "path" "/home/<user>/.local/share/Steam" "label" "" "contentid" … "totalsize" … "update_clean_bytes_tally" … "time_last_update_verified" … "apps" { "<appid>" "<bytes>" … } } "1" { … } }`. Library `"0"` is the Steam root ([local]).
- The legacy layout is `"LibraryFolders" { "TimeNextStatsReport" … "ContentStatsID" … "1" "<path>" }`: numbered string values, and the root is not listed ([pt-libfmt], [pt-fixture]). Accept both forms, take only digit keys, and always add the root itself, as protontricks does (`paths = [steam_path] + library_folders`, [pt-dedupe]).
- Key case varies: `LibraryFolders`/`libraryfolders`, `appid`/`appID` ([pt-acf] "certain appmanifest files … also use 'appID'"). Match keys case-insensitively. Old libraries may use `SteamApps`, and protontricks says "Steam matches library folder paths case-insensitively" ([pt-libs], [pt-steamapps]) **[1]**.
- `appmanifest_<appid>.acf` is KeyValues `"AppState" { "appid" "2694490" "name" "Path of Exile 2" "installdir" "Path of Exile 2" "StateFlags" "4" … }` ([local]). The install dir is `<lib>/steamapps/common/<installdir>` ([pt-acf]). In `StateFlags`, bit 1 means uninstalled and 4 means fully installed ([l-steamflags]) **[1]**.
- The `"apps"` map can be stale. [local]: `appmanifest_2694490.acf` is present with `StateFlags 4`, but `2694490` is missing from `"apps"`. Find installs by listing `appmanifest_*.acf` in each library, not by trusting `"apps"`.
- [local] `<root>/config/libraryfolders.vdf` is an identical copy, ignoring `contentid`. Read `steamapps/libraryfolders.vdf` (protontricks reads that one, [pt-libfmt]).
- Escapes: KeyValues has optional escape sequences (`UsesEscapeSequences`, [kv-esc]). Local Linux files contain none. A shared Windows/Linux parser should still unescape `\\` and `\"`.

### Q3: the Proton prefix

- Proton doesn't pick the location. It exits without `STEAM_COMPAT_DATA_PATH` and builds `CompatData(os.environ["STEAM_COMPAT_DATA_PATH"])` with `prefix_dir = "<that>/pfx/"` ([proton-main], [proton-cd]).
- Which library: Steam passes `STEAM_COMPAT_DATA_PATH`, and "for games installed on an external library, this variable points to that external library" ([s4l-12030], 2025). Also "prefixes are always created in the same library folder as the game" on desktop Linux, but a Steam Deck desktop-mode bug created them in `~/.local/share/Steam/steamapps` ([s4l-8941]). These are user reports in Valve's tracker; I found no Valve doc. protontricks: "Proton prefix and the game installation itself can exist on different Steam libraries". It searches every library and picks the newest `pfx.lock` mtime ([pt-prefix]). Probe `compatdata/2694490` in every library.
- The prefix is created on first launch. [local]: before the first launch there is no `compatdata/2694490`.
- The Wine user is always `steamuser` under Proton:
  - Valve's Wine fork ships `HACK: proton: advapi32: Use steamuser as Wine username` ([vwine-user]).
  - The `proton` script hard-codes `drive_c/users/steamuser/...` and migrates `My Documents` → `Documents`, leaving a `My Documents` symlink ([proton-migrate]).
  - GE-Proton hard-codes the same paths ([ge-proton]).
  - umu makes `drive_c/users/steamuser` and links `drive_c/users/<unix user>` → `steamuser` ([umu-pfx]).
  - Proton's Wine never links profile folders into `$HOME` ([vwine-links]), so `Documents` is a real directory in the prefix.

### Q4: 2315204395 and non-Steam shortcuts

- `2315204395 = 0x89FF372B`, so bit 31 is set. Store app IDs are small (PoE2 = `2694490`: [local], [l-api] `steamid`). Shortcut IDs come from `crc32(exe + appname) | 0x80000000` in Steam ROM Manager, and Heroic copies that function ([srm-appid], [h-sid]). The number is therefore a non-Steam-shortcut compatdata ID. Its origin is GGG's own developer docs, whose "SteamOS Default Location" is `/home/deck/.local/share/Steam/steamapps/compatdata/2315204395/pfx/drive_c/users/steamuser/Documents/My Games/Path of Exile 2/BuildPlanner` ([ggg-dev]).
- Storage: `<root>/userdata/<steamid3>/config/shortcuts.vdf`, binary VDF. Heroic's test expects an empty file to be the bytes `00 "shortcuts" 00 08 08` ([h-vdf-test]). Entries carry `appid` (signed int32), `AppName` and `Exe`, and key case varies. The compatdata folder is `appid & 0xffffffff`. Legacy entries without `appid` fall back to the CRC formula ([pt-shortcut]).
- The ID is per user and per shortcut: it depends on the exe path and name (CRC tools), or differs "each time the game is added" via the Steam UI ([s4l-10184]; [sc-thread] reports GUI IDs that don't match the CRC). It can't be hard-coded.
- Shortcut prefixes land in the root (or default) library: "as far as I can tell it always creates a compatdata at `~/.local/share/Steam/steamapps/compatdata/<nonsteam-appid>`" ([s4l-10184]), and the same claim in [gol-5136]. These are two community sources.
- Recommended: in every library, glob `steamapps/compatdata/*/pfx/drive_c/users/*/Documents/My Games/Path of Exile 2` and `…/pfx/drive_c/Program Files (x86)/Grinding Gear Games/Path of Exile 2`. That costs one `readdir` per library. It needs no binary VDF and stays out of `userdata/`, which is named by Steam account ID, so reading it would expose the account. It also finds `compatdata/2694490`, so report the folder name in `note`.

### Q5: Heroic Games Launcher

- Config dir: `appFolder = join(app.getPath('appData'), 'heroic')` ([h-paths] L7–L23). Electron's `appData` on Linux is `$XDG_CONFIG_HOME` or `~/.config` ([electron]), so it resolves to `~/.config/heroic`. Under Flatpak it is `~/.var/app/com.heroicgameslauncher.hgl/config/heroic` (Flatpak's `XDG_CONFIG_HOME`, [flatpak-xdg]). ProtonUp-Qt uses the same two paths ([pupqt]).
- Files ([h-paths] L28–L39):
  - `config.json`: `{ "defaultSettings": { "winePrefix", "defaultWinePrefixDir", "defaultWinePrefix" (legacy), … }, "version": "v0" }` ([h-cfg]).
  - `GamesConfig/<appName>.json`: `{ "<appName>": { "winePrefix": "…", "wineVersion": { …, "type": "wine"|"proton"|… } }, "version": "v0", "explicit": true }`, written with `JSON.stringify(config, null, 2)` ([h-gc] L36, L145–L147, L336–L367).
- Prefix values may contain `~`, which Heroic replaces with the home dir ([h-gc] L308–L310, [h-cfg] L310–L312). A game with no `winePrefix` falls back to the global `winePrefix`, i.e. the shared prefix ([h-gc] L305).
- Defaults:
  - New-game prefix: `<defaultWinePrefixDir>/<title>`, with `: / * ? < > \ & { } % $ @ ! ™ + ' " ®` and the backtick stripped from the title ([h-ws] L62, [h-rmspecial]). `defaultWinePrefixDir` is `~/Games/Heroic/Prefixes` and the shared prefix is `…/Prefixes/shared` ([h-paths]).
  - Before PR #5336 (2026-04), both the base and the shared prefix were `…/Prefixes/default`, and upgraded installs keep that base ([h-5336]). So `~/Games/Heroic/Prefixes/default/<Title>` and `~/Games/Heroic/Prefixes/<Title>` both occur.
- Proton without umu sets `STEAM_COMPAT_DATA_PATH = winePrefix`, so Wine lives in `<winePrefix>/pfx` ([h-env] L1136–L1141, [h-pfx] L154). With umu the prefix is `<winePrefix>` itself, plus a `pfx -> .` symlink ([umu-pfx] L91–L101). Probe both `drive_c` and `pfx/drive_c`.
- Sideloaded apps: `new Store({ cwd: 'sideload_apps', name: 'library' })` ([h-sl-store]). electron-store joins a relative `cwd` onto `app.getPath('userData')` ([es-cwd]), and the file is tab-indented `JSON.stringify` ([conf-ser]). That gives `<heroic>/sideload_apps/library.json`, also used by [pupqt].
  - Shape ([h-sl-lib] L19–L49): `{ "games": [ { "runner": "sideload", "app_name": "<short-uuid>", "title": "…", "install": { "executable": "<linux path>", "platform": "…", "is_dlc": false }, "folder_name": "<dirname(executable)>", "is_installed": true, … } ] }`.
  - `app_name` is random (`short.generate()`, [h-sd] L130), and its prefix is in `GamesConfig/<app_name>.json` ([h-sd] L202–L208).
- Epic via legendary: `<heroic>/legendaryConfig/legendary/installed.json` ([h-leg]), a map keyed by `app_name` ([h-savesync]) with fields `app_name`, `install_path`, `executable`, `title`, `platform`, … ([h-legtype]). legendary writes it with Python `json.dump(…, indent=2, sort_keys=True)` ([leg-dump]). Python's default `ensure_ascii` turns non-ASCII into `\uXXXX`, with surrogate pairs above U+FFFF. The default install base is `~/Games/Heroic` ([h-paths] L31). PoE2 is sold on Epic ([epic]).
- Flatpak permissions: `--filesystem=~/Games/Heroic:create`, `/mnt`, `/media`, `/run/media`, and `--persist=.` ([fh-heroic] L28–L49). The default prefix paths are real host paths.

### Q6: Lutris

- `CONFIG_DIR` is `~/.config/lutris` if it exists, else `~/.local/share/lutris` ("we're deprecating ~/.config/lutris", [l-settings] L22–L26; added 2023-11-23 in [l-7af680d]). Games live in `<CONFIG_DIR>/games/<slug>-<unixtime>.yml` ([l-config] L21–L31). The DB is `pga_path` from `lutris.conf`, or `~/.local/share/lutris/pga.db` ([l-settings] L46). [local] No `~/.config/lutris`; `~/.local/share/lutris/{games/,lutris.conf,pga.db}`.
- Wine game keys: `game.exe`, `game.prefix`, `game.arch`, `game.working_dir`, `game.args` ([l-wine] L275–L318). With no `prefix`, Lutris uses `$WINEPREFIX` or walks up from the exe to a dir holding `drive_c` + `user.reg`, or a `prefix`/`pfx` child ([l-wine] L705–L714, [l-prefix] L28–L47).
- Installer-made files also hold `name`, `game_slug`, `slug`, `version`, `year`, and `script:`. `script:` is the raw installer: a nested `game:` with unexpanded `$GAMEDIR`, plus `installer:` tasks with their own `prefix:` keys ([l-inst] L330–L364). Only top-level `game:` → `prefix:` counts.
- `pga.db` can be skipped. The yml holds the prefix, the filename holds the slug, and Lutris deletes the yml on uninstall ([l-config] L259–L265). [local] The `games` table holds `configpath`, `slug`, `directory`, `installed` and `runner`, which a path probe doesn't need.
- PoE2 install scripts (lutris.net API; both published scripts, by different authors, [l-api], [l-39535], [l-39016]): runner `wine`, `arch: win64`, `prefix: $GAMEDIR`, `exe: $GAMEDIR/drive_c/Program Files (x86)/Grinding Gear Games/Path of Exile 2/PathOfExile.exe`. `$GAMEDIR` defaults to `<game_path>/<service>/<slug>` with `game_path` defaulting to `~/Games` ([l-target], [l-gamepath]), which gives `~/Games/path-of-exile-2`. The PoE1 script uses the same pattern with `…/Path of Exile/PathOfExile.exe` ([l-api-poe1]).
- Lutris runs Proton through umu by default since 2025-06 ([l-umu]), so the prefix layout is umu's. Desktop integration is off by default, so `Documents` is a real dir ([l-desk]).
- Flatpak `net.lutris.Lutris` has `--filesystem=home` ([fh-lutris]), so recorded paths are host paths. Its config is under `~/.var/app/net.lutris.Lutris/{config,data}/lutris` (inferred from [flatpak-xdg]).

### Q7: Bottles

- `Paths.base = ${XDG_DATA_HOME:-~/.local/share}/bottles` and bottles live in `<base>/bottles/` ([b-paths] L61–L87). Under Flatpak that is `~/.var/app/com.usebottles.bottles/data/bottles/bottles/<name>/` ([flatpak-xdg]). The Flatpak only adds `xdg-data/umu` and `~/Games/umu` ([fh-bottles]).
- A bottle dir is `<bottles>/<config.Path>`, or `config.Path` itself when `Custom_Path` is set, or `<base>/steam/<CompatData>` for Steam-environment bottles ([b-bpath]). That dir is the `WINEPREFIX` ([b-winecmd]; `env.add("WINEPREFIX", bottle)` at L844).
- A custom bottles dir comes from `<base>/data.yml` key `custom_bottles_path` ([b-data], [b-custom]). Custom-location bottles leave `<bottles>/<name>/placeholder.yml` holding `Path: <abs path>` ([b-check] L1437–L1447, [b-ph]).
- `bottle.yml` is PyYAML `SafeDumper` output with indent 4. Keys include `Name`, `Path`, `Custom_Path`, `Runner`, `Environment`, `External_Programs`, `CompatData` ([b-cfg] L165–L225). The locator doesn't need it.
- Wine user: Wine names the profile `$USER`, else the passwd name, else `wine` ([wine-user] L340–L360, exported as `WINEUSERNAME`, [wine-env]). Since 2026-08 ([b-781bb2c], release 67.x), Bottles makes `drive_c/users/<$USER>` and a `steamuser` alias resolve to one profile ([b-alias]). It also replaces Wine's home links with real dirs (`_replace_directory_links`, [b-alias] L194).

### Q8: where things live inside any Wine prefix

- Standalone client: `<prefix>/drive_c/Program Files (x86)/Grinding Gear Games/Path of Exile 2/`, from both Lutris scripts ([l-api]). That is community evidence; I found no GGG statement. The Steam build ships `PathOfExileSteam.exe` and `PathOfExile_x64Steam.exe` ([local]). The scripts launch `PathOfExile.exe`.
- User dir: `<prefix>/drive_c/users/<user>/Documents/My Games/Path of Exile 2`, mirroring GGG's `C:/Users/Name/Documents/My Games/Path of Exile 2/BuildPlanner` ([ggg-dev]). `<user>` is `steamuser` (Proton), the Unix name (Wine), or both via a symlink (umu, Bottles). Glob `drive_c/users/*` minus `Public`, then canonicalise.
- Plain Wine links the profile's `Documents` to `XDG_DOCUMENTS_DIR`, default `$HOME/Documents` ([wine-links]). Proton never does ([vwine-links]), and Lutris and Bottles undo it ([l-desk], [b-alias]). Canonicalising a plain-Wine candidate can therefore land on `~/Documents/My Games/Path of Exile 2`.
- For Proton-style prefixes (Steam, Heroic without umu), also check `<prefix>/pfx/drive_c/...`.

### Q9: line scanner or crates?

- **Heroic JSON: a line scanner is enough.** `JSON.stringify(x, null, 2)` (GamesConfig, config.json, [h-gc], [h-cfg]) and conf's tab-indented form (library.json, [conf-ser]) put every string-valued member on one line: `"winePrefix": "…"`. JSON.stringify escapes only `"`, `\`, control characters and lone surrogates; non-ASCII stays raw ([ecma-json]). So the scanner needs an exact key match plus a ~20-line unescape (`\" \\ \/ \b \f \n \r \t \uXXXX`, joining surrogate pairs for legendary's ASCII-only output) and a leading-`~` expansion. serde_json (1.0.154, 2026-10-11, [c-serde-json]) is maintained, but this doesn't need it.
- **Lutris YAML: a naive scan is wrong.** Lutris writes `yaml.safe_dump(config, default_flow_style=False)` ([l-yaml]). With PyYAML's defaults on synthetic data ([local] experiment):
  - Long plain scalars fold at a space onto a more-indented line. The default PoE2 exe path breaks after `Grinding Gear`.
  - Non-ASCII forces double quotes with `\xC9`-style escapes, and long ones continue with a trailing `\` and a leading `\ `.
  - `'` forces single quotes with `''`.
  - Type-like strings (`yes`, `123`) are quoted.
  - The duplicated `script:` subtree (above) means a scan must stay inside top-level `game:`.

  A scoped reader that takes only `game.prefix` (plain, single- and double-quoted scalars, continuation lines) is about 100 lines plus fixtures generated once with PyYAML. Bottles' `placeholder.yml` and `data.yml` hold one top-level scalar each and need the same reader. Bottles uses libyaml's `CSafeDumper`; that it folds the same way is inferred, not tested.
- YAML crates:

| Crate | Status (crates.io / RustSec, 2026-10) |
|---|---|
| `serde_yaml` | `0.9.34+deprecated`, repo archived 2024-03-25 ([c-serde-yaml], [serde-yaml-gh]) — not maintained |
| `serde_yml` | "DEPRECATED … unmaintained" shim; RUSTSEC-2025-0068 "unsound and unmaintained" ([c-serde-yml], [rs-0068]) |
| `yaml-rust` | RUSTSEC-2024-0320 unmaintained ([rs-0320]) |
| `serde_norway` | 0.9.42 (2024-12-21); RustSec lists it as the maintained `serde_yaml` fork (libyaml-based) ([c-norway], [rs-0068]) |
| `yaml-rust2` | 0.13.0, 2026-09-11, pure Rust ([c-yaml-rust2]) |
| `saphyr` / `serde-saphyr` | 0.2.2 (2026-10-15) / 2.0.0 (2026-10-13, "panic-free parsing") ([c-saphyr], [c-serde-saphyr]) |

- Recommendation: std only. Use a line scanner for Heroic JSON, and a scoped scalar reader for Lutris `game.prefix` and the Bottles `Path`/`custom_bottles_path`. Don't parse `game.exe`; probe the standard install path inside the prefix. If YAML needs grow, use `yaml-rust2` or `saphyr`, never `serde_yaml` or `serde_yml`.

### Probe recipe that follows (recommendation)

1. Steam roots, in this order: `~/.steam/steam`, `~/.steam/root`, `${XDG_DATA_HOME:-~/.local/share}/Steam`, `~/.steam/debian-installation`, `~/Steam`, then Flatpak and Snap (incl. `~/.snap/data/...`). Canonicalise and dedupe, giving Source `Steam` or `FlatpakSteam`.
2. For each root: the root itself plus `libraryfolders.vdf` paths (Flatpak remap first). Per library, canonicalise `steamapps` (any case) and dedupe. Read `appmanifest_2694490.acf` → `common/<installdir>`. Glob `compatdata/*/pfx/drive_c/...` (Source `ProtonPrefix`; note store vs shortcut).
3. Heroic: every `winePrefix` in `GamesConfig/*.json` and `config.json`, plus the fallback globs `~/Games/Heroic/Prefixes/*` and `…/Prefixes/default/*`. Lutris: top-level `game.prefix` of every `games/*.yml` in both dirs. Bottles: `bottles/*`, placeholders and the custom dir. Fallbacks: `~/.wine`, `~/Games/umu/*`.
4. Every prefix: `{.,pfx}/drive_c/users/*/Documents/My Games/Path of Exile 2` and `{.,pfx}/drive_c/Program Files (x86)/Grinding Gear Games/Path of Exile 2`. Canonicalise, dedupe, and rank by Client.txt mtime.

## Conflicting

- **GGG docs vs the Steam app ID.** GGG's SteamOS example uses `compatdata/2315204395` ([ggg-dev]), but a Steam store install gets `compatdata/2694490` ([pt-prefix]). The GGG path only fits a non-Steam-shortcut install, likely the standalone client. Two guides copied it.
- **How shortcut IDs are made.** Tools use `crc32(exe+name)|0x80000000` ([srm-appid], [h-sid], [pt-shortcut]), so IDs are always ≥ 2^31 and 10 digits. [s4l-10184] describes Steam-UI IDs as "random" and "always 9 digits". [sc-thread] (2022) first saw no CRC match, then wrote "I made a mistake … steam itself does create the ids differently". Don't filter compatdata by ID range.
- **compatdata placement.** The game's library ([s4l-12030], [s4l-8941] desktop) vs the root library ([s4l-8941] Deck desktop-mode bug; non-Steam shortcuts per [s4l-10184], [gol-5136]). [s4l-9682] reports compatdata being deleted when a move fails, which implies it moves with the game. Hence probe all libraries.
- **Flatpak library-0 string.** It can be `$HOME/.local/share/Steam` (what protontricks remaps, [pt-libs]) or `$HOME/.var/app/com.valvesoftware.Steam/.local/share/Steam` (what `FLATPAK_STEAM_XDG_DIRS_PREFIX` yields, [fh-yml]). The wrapper keeps an older prefix found via `~/.steam/root` ([fh-wrap] L363–L373). Handle both.
- **Snap docs vs code.** The snap docs say the default library "points to `~/snap/steam/common/.local/share/steam`" (lowercase, [snap-docs]). The code gives `$XDG_DATA_HOME/Steam` ([snap-launch], [launcher]), and protontricks uses `Steam` ([pt-dirs]). Trust the code.
- **Heroic defaults moved.** It went from `Prefixes/default` (base and shared) to `Prefixes/<Title>` with `Prefixes/shared` ([h-5336]). Both layouts exist on disk.
- **Lutris config dir.** `~/.config/lutris` (installs predating 2023-11, still honoured) vs `~/.local/share/lutris` (new) ([l-7af680d]). Check both.
- **Wine user name in non-Steam prefixes.** It is `$USER` for Wine ([wine-user]) and `steamuser` for Proton ([vwine-user]). umu and Bottles ≥ 67.x alias the two ([umu-pfx], [b-alias]). Older Bottles and Heroic prefixes may have only one.

## Unknown

- PoE2's Epic `app_name` and install folder name in legendary's `installed.json` (no install to inspect; [epic] confirms availability only).
- Whether the Snap's `libraryfolders.vdf` records host paths. Snap remaps no paths and changes only `$HOME`, so it is probably fine; not observed. `src/steam-snap.sh`, which symlinks a native `steamapps` into the snap root ([snap-adopt]), is copied into the snap but I found no command chain that runs it. Canonicalising `steamapps` covers it either way.
- Flatpak Steam libraries outside `/mnt`, `/media` and `/run/media` added through user overrides: is a path under `$HOME` real or persisted? Recommended: try the raw path, then `~/.var/app/com.valvesoftware.Steam/<rel>`.
- Heroic Flatpak prefixes outside `~/Games/Heroic` (e.g. `~/WinePrefixes/x`) presumably live under `~/.var/app/com.heroicgameslauncher.hgl/<rel>` because of `--persist=.` ([fh-heroic], [flatpak-perm]). Inferred, not tested.
- Bottles Flatpak paths recorded as `/var/data/...` (when the home path has Windows-unsafe components, [b-paths] L41–L58) map to `~/.var/app/com.usebottles.bottles/data/...` ([flatpak-perm]). Inferred.
- Whether the standalone installer's default is still `Program Files (x86)\Grinding Gear Games\Path of Exile 2` (community scripts only), and which exe names it installs.
- Users can override `STEAM_COMPAT_DATA_PATH` in launch options; we can't see that without reading `localconfig.vdf` under `userdata/`, which we exclude.
- Other launchers (Faugus, raw umu with custom `WINEPREFIX`, CrossOver bottles) are not covered beyond the `~/.wine` and `~/Games/umu/*` fallbacks.
- Whether to probe `$XDG_DOCUMENTS_DIR/My Games/Path of Exile 2` directly, outside any prefix. Only plain-Wine Documents links lead there; ADR 0003 decides not to probe it.

[ggg-dev]: https://www.pathofexile.com/developer/docs/game
[launcher]: https://repo.steampowered.com/steam/archive/stable/steam_latest.tar.gz
[deb-steam]: https://ftp.debian.org/debian/pool/contrib/s/steam-installer/
[deb-bug]: https://alioth-lists.debian.net/pipermail/pkg-games-devel/2019-January/050676.html
[fh-yml]: https://github.com/flathub/com.valvesoftware.Steam/blob/6e7846ff7743da4ae2ea28fbfc7ba117182610e8/com.valvesoftware.Steam.yml#L42-L68
[fh-wrap]: https://github.com/flathub/com.valvesoftware.Steam/blob/6e7846ff7743da4ae2ea28fbfc7ba117182610e8/steam_wrapper/steam_wrapper.py#L220-L373
[flatpak-perm]: https://docs.flatpak.org/en/latest/sandbox-permissions.html
[flatpak-xdg]: https://docs.flatpak.org/en/latest/conventions.html
[snap-yml]: https://github.com/canonical/steam-snap/blob/b13597e5be9b39a90447b5932a2b401bec1a7af0/snap/snapcraft.yaml#L548-L551
[snap-launch]: https://github.com/canonical/steam-snap/blob/b13597e5be9b39a90447b5932a2b401bec1a7af0/src/desktop-launch#L243-L246
[snap-howto]: https://github.com/canonical/steam-snap/blob/b13597e5be9b39a90447b5932a2b401bec1a7af0/docs/howto/troubleshooting/game-workarounds.md
[snap-docs]: https://github.com/canonical/steam-snap/blob/b13597e5be9b39a90447b5932a2b401bec1a7af0/docs/reference/external-libraries.md
[snap-adopt]: https://github.com/canonical/steam-snap/blob/b13597e5be9b39a90447b5932a2b401bec1a7af0/src/steam-snap.sh
[snap-hidden]: https://forum.snapcraft.io/t/experimental-flag-for-hiding-snap/28509
[steamlocate]: https://github.com/WilliamVenner/steamlocate-rs/blob/d740b423aee82e8e742595d0cdb4edcc91c24216/src/locate/linux.rs#L5-L39
[pt-dirs]: https://github.com/Matoking/protontricks/blob/23f19063999bb016fc8e1be048b76e4c5e4c78dc/src/protontricks/steam.py#L29-L45
[pt-steamapps]: https://github.com/Matoking/protontricks/blob/23f19063999bb016fc8e1be048b76e4c5e4c78dc/src/protontricks/steam.py#L399-L418
[pt-find]: https://github.com/Matoking/protontricks/blob/23f19063999bb016fc8e1be048b76e4c5e4c78dc/src/protontricks/steam.py#L421-L507
[pt-acf]: https://github.com/Matoking/protontricks/blob/23f19063999bb016fc8e1be048b76e4c5e4c78dc/src/protontricks/steam.py#L270-L320
[pt-prefix]: https://github.com/Matoking/protontricks/blob/23f19063999bb016fc8e1be048b76e4c5e4c78dc/src/protontricks/steam.py#L1070-L1114
[pt-libs]: https://github.com/Matoking/protontricks/blob/23f19063999bb016fc8e1be048b76e4c5e4c78dc/src/protontricks/steam.py#L1179-L1199
[pt-libfmt]: https://github.com/Matoking/protontricks/blob/23f19063999bb016fc8e1be048b76e4c5e4c78dc/src/protontricks/steam.py#L1260-L1319
[pt-dedupe]: https://github.com/Matoking/protontricks/blob/23f19063999bb016fc8e1be048b76e4c5e4c78dc/src/protontricks/steam.py#L1330-L1336
[pt-shortcut]: https://github.com/Matoking/protontricks/blob/23f19063999bb016fc8e1be048b76e4c5e4c78dc/src/protontricks/steam.py#L1532-L1598
[pt-fixture]: https://github.com/Matoking/protontricks/blob/23f19063999bb016fc8e1be048b76e4c5e4c78dc/tests/conftest.py#L466-L483
[proton-cd]: https://github.com/ValveSoftware/Proton/blob/bc896217ffcd67f892f952b010f898fe7bd0f0a1/proton#L632-L645
[proton-migrate]: https://github.com/ValveSoftware/Proton/blob/bc896217ffcd67f892f952b010f898fe7bd0f0a1/proton#L955-L986
[proton-main]: https://github.com/ValveSoftware/Proton/blob/bc896217ffcd67f892f952b010f898fe7bd0f0a1/proton#L2137-L2147
[vwine-user]: https://github.com/ValveSoftware/wine/commit/59b20e82fd1211314e234b08901fafe10490ff94
[vwine-links]: https://github.com/ValveSoftware/wine/commit/d6c34d05dd44c1bb4364d1611bbc4b12ea9e7df3
[ge-proton]: https://github.com/GloriousEggroll/proton-ge-custom/blob/master/proton#L68-L72
[s4l-12030]: https://github.com/ValveSoftware/steam-for-linux/issues/12030
[s4l-8941]: https://github.com/ValveSoftware/steam-for-linux/issues/8941
[s4l-9682]: https://github.com/ValveSoftware/steam-for-linux/issues/9682
[s4l-10184]: https://github.com/ValveSoftware/steam-for-linux/issues/10184
[gol-5136]: https://www.gamingonlinux.com/forum/topic/5136
[sc-thread]: https://steamcommunity.com/discussions/forum/1/3361398061434096995
[srm-appid]: https://github.com/SteamGridDB/steam-rom-manager/blob/master/src/lib/helpers/steam/generate-app-id.ts
[h-sid]: https://github.com/Heroic-Games-Launcher/HeroicGamesLauncher/blob/509a34feea8541b73c928e289f2cf262d432db4a/src/backend/shortcuts/nonesteamgame/steamhelper.ts#L198-L224
[h-vdf-test]: https://github.com/Heroic-Games-Launcher/HeroicGamesLauncher/blob/509a34feea8541b73c928e289f2cf262d432db4a/src/backend/shortcuts/nonesteamgame/__tests__/nonesteamgame.test.ts#L131-L148
[h-paths]: https://github.com/Heroic-Games-Launcher/HeroicGamesLauncher/blob/509a34feea8541b73c928e289f2cf262d432db4a/src/backend/constants/paths.ts#L7-L39
[h-gc]: https://github.com/Heroic-Games-Launcher/HeroicGamesLauncher/blob/509a34feea8541b73c928e289f2cf262d432db4a/src/backend/game_config.ts#L36-L367
[h-cfg]: https://github.com/Heroic-Games-Launcher/HeroicGamesLauncher/blob/509a34feea8541b73c928e289f2cf262d432db4a/src/backend/config.ts#L300-L357
[h-ws]: https://github.com/Heroic-Games-Launcher/HeroicGamesLauncher/blob/509a34feea8541b73c928e289f2cf262d432db4a/src/frontend/screens/Library/components/InstallModal/WineSelector/index.tsx#L54-L63
[h-rmspecial]: https://github.com/Heroic-Games-Launcher/HeroicGamesLauncher/blob/509a34feea8541b73c928e289f2cf262d432db4a/src/frontend/helpers/index.ts#L119-L122
[h-sd]: https://github.com/Heroic-Games-Launcher/HeroicGamesLauncher/blob/509a34feea8541b73c928e289f2cf262d432db4a/src/frontend/screens/Library/components/InstallModal/SideloadDialog/index.tsx#L128-L210
[h-sl-store]: https://github.com/Heroic-Games-Launcher/HeroicGamesLauncher/blob/509a34feea8541b73c928e289f2cf262d432db4a/src/backend/storeManagers/sideload/electronStores.ts#L1-L7
[h-sl-lib]: https://github.com/Heroic-Games-Launcher/HeroicGamesLauncher/blob/509a34feea8541b73c928e289f2cf262d432db4a/src/backend/storeManagers/sideload/library.ts#L19-L76
[h-env]: https://github.com/Heroic-Games-Launcher/HeroicGamesLauncher/blob/509a34feea8541b73c928e289f2cf262d432db4a/src/backend/launcher.ts#L1110-L1147
[h-pfx]: https://github.com/Heroic-Games-Launcher/HeroicGamesLauncher/blob/509a34feea8541b73c928e289f2cf262d432db4a/src/backend/utils.ts#L142-L156
[h-leg]: https://github.com/Heroic-Games-Launcher/HeroicGamesLauncher/blob/509a34feea8541b73c928e289f2cf262d432db4a/src/backend/storeManagers/legendary/constants.ts#L4-L10
[h-legtype]: https://github.com/Heroic-Games-Launcher/HeroicGamesLauncher/blob/509a34feea8541b73c928e289f2cf262d432db4a/src/common/types/legendary.ts#L14-L33
[h-savesync]: https://github.com/Heroic-Games-Launcher/HeroicGamesLauncher/blob/509a34feea8541b73c928e289f2cf262d432db4a/src/backend/save_sync.ts#L48-L58
[leg-dump]: https://github.com/derrod/legendary/blob/42f6bdeadde3a9526dc8eb713763476999ac217b/legendary/lfs/lgndry.py#L364-L365
[h-5336]: https://github.com/Heroic-Games-Launcher/HeroicGamesLauncher/commit/1285094d085a4856a0e1f5ed190dca023484e684
[fh-heroic]: https://github.com/flathub/com.heroicgameslauncher.hgl/blob/1b6bc9882ce765087c86f2b568188a03093ad358/com.heroicgameslauncher.hgl.yml#L28-L49
[electron]: https://www.electronjs.org/docs/latest/api/app#appgetpathname
[es-cwd]: https://github.com/sindresorhus/electron-store/blob/v8.2.0/index.js#L60-L64
[conf-ser]: https://github.com/sindresorhus/conf/blob/v10.2.0/source/index.ts#L430
[pupqt]: https://github.com/DavidoTek/ProtonUp-Qt/blob/main/pupgui2/heroicutil.py#L15-L23
[epic]: https://store.epicgames.com/all-dlc/path-of-exile-2
[umu-pfx]: https://github.com/Open-Wine-Components/umu-launcher/blob/e2b203a1fdd2af9f35166f5713cb3f85d72587e2/umu/umu_run.py#L79-L120
[umu-default]: https://github.com/Open-Wine-Components/umu-launcher/blob/e2b203a1fdd2af9f35166f5713cb3f85d72587e2/umu/umu_run.py#L145-L162
[l-settings]: https://github.com/lutris/lutris/blob/1f32fe0e46e2a5d92fad9db2846c7cb7becd20c3/lutris/settings.py#L22-L46
[l-7af680d]: https://github.com/lutris/lutris/commit/7af680d2655076e5ea536c754d3a681262520b9b
[l-config]: https://github.com/lutris/lutris/blob/1f32fe0e46e2a5d92fad9db2846c7cb7becd20c3/lutris/config.py#L21-L265
[l-inst]: https://github.com/lutris/lutris/blob/1f32fe0e46e2a5d92fad9db2846c7cb7becd20c3/lutris/installer/installer.py#L330-L364
[l-wine]: https://github.com/lutris/lutris/blob/1f32fe0e46e2a5d92fad9db2846c7cb7becd20c3/lutris/runners/wine.py#L275-L714
[l-desk]: https://github.com/lutris/lutris/blob/1f32fe0e46e2a5d92fad9db2846c7cb7becd20c3/lutris/runners/wine.py#L318-L327
[l-prefix]: https://github.com/lutris/lutris/blob/1f32fe0e46e2a5d92fad9db2846c7cb7becd20c3/lutris/util/wine/prefix.py#L28-L47
[l-yaml]: https://github.com/lutris/lutris/blob/1f32fe0e46e2a5d92fad9db2846c7cb7becd20c3/lutris/util/yaml.py#L27-L28
[l-target]: https://github.com/lutris/lutris/blob/1f32fe0e46e2a5d92fad9db2846c7cb7becd20c3/lutris/installer/interpreter.py#L120-L127
[l-gamepath]: https://github.com/lutris/lutris/blob/1f32fe0e46e2a5d92fad9db2846c7cb7becd20c3/lutris/sysoptions.py#L106-L114
[l-steamflags]: https://github.com/lutris/lutris/blob/1f32fe0e46e2a5d92fad9db2846c7cb7becd20c3/docs/steam.rst
[l-umu]: https://github.com/lutris/lutris/commit/ef6d6a609b59b375c455ba125f4f06ef4cded432
[l-api]: https://lutris.net/api/installers/path-of-exile-2
[l-api-poe1]: https://lutris.net/api/installers/path-of-exile
[l-39535]: https://lutris.net/games/install/39535/view
[l-39016]: https://lutris.net/games/install/39016/view
[fh-lutris]: https://github.com/flathub/net.lutris.Lutris/blob/master/net.lutris.Lutris.yml#L20-L27
[b-paths]: https://github.com/bottlesdevs/Bottles/blob/cfd96650a929fcafef1fc2b3e6cd23e5b144a44d/bottles/backend/globals.py#L41-L87
[b-bpath]: https://github.com/bottlesdevs/Bottles/blob/cfd96650a929fcafef1fc2b3e6cd23e5b144a44d/bottles/backend/utils/manager.py#L96-L103
[b-winecmd]: https://github.com/bottlesdevs/Bottles/blob/cfd96650a929fcafef1fc2b3e6cd23e5b144a44d/bottles/backend/wine/winecommand.py#L557-L562
[b-custom]: https://github.com/bottlesdevs/Bottles/blob/cfd96650a929fcafef1fc2b3e6cd23e5b144a44d/bottles/backend/managers/manager.py#L182-L197
[b-check]: https://github.com/bottlesdevs/Bottles/blob/cfd96650a929fcafef1fc2b3e6cd23e5b144a44d/bottles/backend/managers/manager.py#L1418-L1452
[b-ph]: https://github.com/bottlesdevs/Bottles/blob/cfd96650a929fcafef1fc2b3e6cd23e5b144a44d/bottles/backend/managers/manager.py#L1992-L1997
[b-data]: https://github.com/bottlesdevs/Bottles/blob/cfd96650a929fcafef1fc2b3e6cd23e5b144a44d/bottles/backend/managers/data.py#L29-L49
[b-cfg]: https://github.com/bottlesdevs/Bottles/blob/cfd96650a929fcafef1fc2b3e6cd23e5b144a44d/bottles/backend/models/config.py#L165-L225
[b-alias]: https://github.com/bottlesdevs/Bottles/blob/cfd96650a929fcafef1fc2b3e6cd23e5b144a44d/bottles/backend/utils/wine.py#L41-L135
[b-781bb2c]: https://github.com/bottlesdevs/Bottles/commit/781bb2cc663633f55c3ab68f333da8df31ff8515
[fh-bottles]: https://github.com/flathub/com.usebottles.bottles/blob/ab194a53a873cb1c04279c0a05fa4d760dd372f9/com.usebottles.bottles.yml#L22-L23
[wine-user]: https://github.com/wine-mirror/wine/blob/840a939ee526137639665ff5f587224d6a36403f/dlls/ntdll/unix/loader.c#L340-L360
[wine-cfg]: https://github.com/wine-mirror/wine/blob/840a939ee526137639665ff5f587224d6a36403f/dlls/ntdll/unix/loader.c#L362-L380
[wine-env]: https://github.com/wine-mirror/wine/blob/840a939ee526137639665ff5f587224d6a36403f/dlls/ntdll/unix/env.c#L1052
[wine-links]: https://github.com/wine-mirror/wine/blob/840a939ee526137639665ff5f587224d6a36403f/dlls/shell32/shellpath.c#L2822-L2846
[kv-esc]: https://github.com/ValveSoftware/source-sdk-2013/blob/master/src/tier1/KeyValues.cpp#L628-L631
[ecma-json]: https://tc39.es/ecma262/#sec-quotejsonstring
[c-serde-json]: https://crates.io/crates/serde_json
[c-serde-yaml]: https://crates.io/crates/serde_yaml
[serde-yaml-gh]: https://github.com/dtolnay/serde-yaml
[c-serde-yml]: https://crates.io/crates/serde_yml
[c-norway]: https://crates.io/crates/serde_norway
[c-yaml-rust2]: https://crates.io/crates/yaml-rust2
[c-saphyr]: https://crates.io/crates/saphyr
[c-serde-saphyr]: https://crates.io/crates/serde-saphyr
[rs-0068]: https://rustsec.org/advisories/RUSTSEC-2025-0068.html
[rs-0320]: https://rustsec.org/advisories/RUSTSEC-2024-0320.html
