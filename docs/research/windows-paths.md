# Windows paths: how a passive reader finds PoE2's files

Researched 2026-10-23 for the `locate` module. Each claim links to its source. **[1 source]** marks
a claim that rests on a single source. "Format evidence" means another tool's docs or tests, read but not copied.
Third-party mirrors used: `api.steamcmd.net` (Valve PICS app info) and `api.egdata.app` (Epic catalog).
All paths and IDs in examples are synthetic except published product IDs.

## Confirmed

| Item | Path / key / format | Sources |
|---|---|---|
| PoE2 Steam store app ID | `2694490` | [store], [appdetails], [pics], [pcgw] |
| What `2315204395` is | Not a store app ID. `0x89FF372B` has the high bit set and does not fit Steam's 24-bit app-ID field. Valve requires that bit for non-Steam-shortcut IDs. It appears in GGG's SteamOS BuildPlanner path. | [sdk-gameid], [ggg-docs], [heroic-id] |
| Steam root, per user | `HKCU\Software\Valve\Steam`, value `SteamPath` (REG_SZ), forward slashes, reported lower-case, e.g. `c:/program files (x86)/steam` | [deadloader], [vibeshine] |
| Steam root, machine | `HKLM\SOFTWARE\WOW6432Node\Valve\Steam`, value `InstallPath` (REG_SZ), backslashes, original case. On 32-bit Windows: `HKLM\SOFTWARE\Valve\Steam`. | [sdk-scripts], [steamlocate-win], [traxile], [vibeshine], [steambrew] |
| Steam library list | `<SteamRoot>\steamapps\libraryfolders.vdf` and `<SteamRoot>\config\libraryfolders.vdf`, in KeyValues text with `\\` in paths | [steamlocate-lib], [protontricks], [lutris-cfg], [forum-2copies], [forum-vdf] |
| Steam app manifest | `<Library>\steamapps\appmanifest_2694490.acf`, key `AppState` → `installdir`; the game is in `<Library>\steamapps\common\<installdir>` | [steamlocate-lib], [steamlocate-acf] |
| PoE2 Steam folder and exe | `installdir` = `Path of Exile 2`; launch `PathOfExileSteam.exe --nopatch` | [pics] |
| Standalone default install | `C:\Program Files (x86)\Grinding Gear Games\Path of Exile 2\PathOfExile.exe`; the user can choose another folder | [forum-opt], [forum-dir], [lutris-poe2] |
| Standalone registry | `HKCU\Software\GrindingGearGames\Path of Exile 2`, value `InstallLocation` **[1 source]**. PoE1 uses the same pattern. | [forum-reg], [traxile] |
| Sold on Epic? | Yes, since Early Access began (2024-12-06) | [ggg-eafaq], [epic-content], [pcgw] |
| Epic install manifests | `%PROGRAMDATA%\Epic\EpicGamesLauncher\Data\Manifests\*.item` (JSON, one per install) | [legendary-lfs], [epic-doc], [jayd] |
| Epic install list | `%PROGRAMDATA%\Epic\UnrealEngineLauncher\LauncherInstalled.dat` (JSON `InstallationList[]`) | [bottles-egs] |
| Recognise PoE2 on Epic | `CatalogNamespace` = `58a18be6bbba41a0bf52b014bb81d33b`. `AppName` is `57d39134358849b1b765ee649ea6a6e9` for the main build and `54c5e100caf049bf88068afe4c02de21` for the Free Weekend build **[1 source]**. | [epic-content], [egdata-item], [egdata-fw] |
| Epic folder name | `FolderName` = `PathOfExile2`, under Epic's default root `C:\Program Files\Epic Games\` **[1 source each]** | [egdata-item], [epic-help] |
| Documents folder | `SHGetKnownFolderPath(FOLDERID_Documents = {FDD39AD0-238F-46AF-ADB4-6C85480369C7})`. It follows OneDrive Known Folder Move and returns no trailing `\`. | [ms-kfid], [ms-shgkfp], [ms-kfm] |
| Registry mirror (do not read) | `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\User Shell Folders`, value `Personal` (REG_EXPAND_SZ) | [ms-personal], [chen] |
| PoE2 user dir | `<Documents>\My Games\Path of Exile 2`; the same path for Steam, standalone and Epic | [pcgw], [ggg-docs], [neversink], [forum-cfg] |
| Game config | `<user dir>\poe2_production_Config.ini` | [pcgw], [forum-cfg], [exileui-cfg] |
| Item filters | `<user dir>\*.filter` in the folder itself, not a subfolder. `<user dir>\OnlineFilters\` holds filters followed through the website. | [neversink], [forum-filter], [exileui-lf] |
| Build planner | `<user dir>\BuildPlanner\*.build` (JSON, a single `Build` object, version 1, experimental) | [ggg-docs] |
| Rust crates | Recommended: `windows-sys` 0.61 only (2 crates on Windows, 0 on Linux). See Q7. | [cr-api], crate sources |
| Client.txt (out of scope) | `<install dir>\logs\Client.txt`. Several tools agree; I found no GGG statement. | [poe2mt], [splitstone], [juice] |

### Q1: the Steam app ID

- **2694490 is the store app.** It is in the store URL, in `steam_appid` from Valve's `appdetails` API ("Path of
  Exile 2", Grinding Gear Games, Windows only) and in PICS (`installdir`, launch config, depots 2694491–2694493)
  ([store], [appdetails], [pics]). PCGamingWiki lists Steam `2694490` ([pcgw]).
- **2315204395 cannot be a store app ID.** Valve's `CGameID` stores the app ID in a 24-bit field
  (`unsigned int m_nAppID : 24`), so the maximum is 16,777,215 ([sdk-gameid] L1085). The largest PoE2 DLC ID in
  PICS is 4,619,890 ([pics]).
- **It has the form of a shortcut ID.** Valve accepts a shortcut game ID only if its 32-bit ID has `0x80000000`
  set and is at least `5000 | 0x80000000`. The SDK comment says "shortcuts are pushed beyond that range"
  ([sdk-gameid] L1057–1060). `2315204395` = `0x89FF372B` meets that rule. As format evidence, Heroic (and Steam ROM
  Manager, which it credits) computes shortcut IDs as `crc32(exe + name) | 0x80000000`. Heroic stores the result as
  a signed int32 in `shortcuts.vdf` and uses the unsigned form for artwork ([heroic-id] L199–223).
- **Where it comes from.** GGG's own developer docs give the SteamOS BuildPlanner path as
  `/home/<user>/.local/share/Steam/steamapps/compatdata/2315204395/pfx/drive_c/users/steamuser/Documents/My Games/Path of Exile 2/BuildPlanner`
  (GGG writes the Steam Deck default user) ([ggg-docs]). One guide copies this path ([guide-2315]); others use
  `2694490` ([guide-2694], [neversink]).
- **Implication.** Search `compatdata/2694490` for the Steam build. Find other prefixes by scanning, never by a
  hardcoded shortcut ID. [linux-paths.md](linux-paths.md) covers the details.

### Q2: Steam on Windows

**Registry (read order, every hit kept as a candidate):**
1. `HKCU\Software\Valve\Steam` → `SteamPath`. The Steam client writes this per-user value with forward slashes and
   in lower case, e.g. `c:/program files (x86)/steam` ([deadloader], [vibeshine]). Normalise `/` to `\` and compare
   case-insensitively, as Windows does ([ms-naming]). A process running as SYSTEM sees SYSTEM's HKCU and finds no
   key ([vibeshine]).
2. `HKLM\SOFTWARE\WOW6432Node\Valve\Steam` → `InstallPath`, e.g. `C:\Program Files (x86)\Steam`, with backslashes
   and original case ([vibeshine], [steambrew], [steamlocate-win] L15–28). Valve: "Steam writes to the Wow6432Node
   path in the registry" ([sdk-scripts]).
3. `HKLM\SOFTWARE\Valve\Steam` → `InstallPath`, for 32-bit Windows ([steamlocate-win], [traxile] L12–13).
4. Fallback guess `C:\Program Files (x86)\Steam` (`Source::Fallback`).

**`libraryfolders.vdf`, current layout** (synthetic; shape as in [forum-sample]). Entry `"0"` is always the main Steam
install ([forum-vdf]). Each entry has `path` and an `apps` map of app ID → size ([steamlocate-lib] L20–45):

```
"libraryfolders"
{
	"contentstatsid"		"-1111111111111111111"
	"0"
	{
		"path"		"C:\\Program Files (x86)\\Steam"
		"label"		""
		"contentid"		"2222222222222222222"
		"totalsize"		"0"
		"update_clean_bytes_tally"		"0"
		"time_last_update_corruption"		"0"
		"apps"
		{
			"2694490"		"150469730160"
		}
	}
	"1"
	{
		"path"		"D:\\SteamLibrary"
		"label"		""
	}
}
```

**Older layouts** that a parser must accept, with both forms in one synthetic file (format evidence, [protontricks]
L471–475 and L754–768, [lutris-cfg] L277):

```
"LibraryFolders"
{
	"TimeNextStatsReport"		"1234567890"
	"ContentStatsID"		"-1111111111111111111"
	"1"		"D:\\SteamLibrary"
	"2"
	{
		"path"		"E:\\Games\\Steam"
		"label"		""
		"mounted"		"1"
	}
}
```

Parser rules backed by the sources above:
- Match keys case-insensitively. Both `libraryfolders` and `LibraryFolders` occur ([lutris-cfg] L222, [protontricks]).
- Only numeric keys are libraries. Skip `contentstatsid`, `TimeNextStatsReport` and similar keys ([lutris-cfg] L231–232).
- A library value is either a plain string (legacy) or an object with `path`. If `mounted` is present and not `"1"`,
  skip the library ([lutris-cfg] L277–285).
- Windows paths are written with doubled backslashes ([forum-vdf], [forum-sample], [steamlocate-acf] L5). Valve's
  KeyValues escapes are `\n`, `\t`, `\\` and `\"` ([vdc-kv])
  **[1 source for `\"`]**. Windows file names cannot contain `"` ([ms-naming]). A `\"` can
  therefore appear only in text such as `label`, but the parser should still accept it.
- Both files can list the same library, and `SteamPath` and `InstallPath` can name one root in different case.
  Remove duplicates case-insensitively ([ms-naming]).
- Each extra library root also has a `libraryfolder.vdf` marker (singular) holding `contentid` and `label`
  ([steamlocate-marker]). It is not needed for discovery.
- The `apps` map can be stale. List `steamapps\appmanifest_*.acf` per library instead ([steamlocate-lib] L126–130).

**`appmanifest_2694490.acf`** (synthetic; layout as in [steamlocate-acf]). The game is in
`<Library>\steamapps\common\<installdir>` ([steamlocate-lib] L251–256):

```
"AppState"
{
	"appid"		"2694490"
	"Universe"		"1"
	"name"		"Path of Exile 2"
	"StateFlags"		"4"
	"installdir"		"Path of Exile 2"
	"LastUpdated"		"1790000000"
}
```

Key case varies between files (`LastUpdated` vs `lastupdated`), so look keys up case-insensitively
([steamlocate-acf], other fixtures in the same directory). PoE2's `installdir` is `Path of Exile 2` and its launch
executable is `PathOfExileSteam.exe` with `--nopatch` ([pics]). The default Steam install is therefore
`C:\Program Files (x86)\Steam\steamapps\common\Path of Exile 2` ([poe2mt]).

### Q3: standalone GGG installer

- The installer is `PathOfExile2Installer.exe`, served from `web.poecdn.com/protected/downloads-poe2/`. The game
  executable is `Program Files (x86)/Grinding Gear Games/Path of Exile 2/PathOfExile.exe` ([lutris-poe2]).
- Default folder: `C:\Program Files (x86)\Grinding Gear Games\Path of Exile 2`. Two players report it, one of them
  correcting another's "Program Files" to "(x86)" ([forum-opt], [forum-dir]). GGG staff: "To select the install
  location in the Path of Exile 2 installer, be sure to select the 'Options' button" ([forum-opt]). Players say the
  folder is portable, so they copy it to another drive and run `PathOfExile.exe` from there ([forum-dir]).
- Registry: to move an install, a player says to edit `InstallLocation` under
  `HKEY_CURRENT_USER\SOFTWARE\GrindingGearGames\Path of Exile 2` ([forum-reg]) **[1 source]**. Format evidence: TraXile
  reads PoE1's `HKCU\SOFTWARE\GrindingGearGames\Path of Exile` → `InstallLocation`, then a `Wow6432Node` variant
  ([traxile] L14–15, L36–46).
- **Implication.** Treat the registry value as a hint. A copied folder leaves it stale, so check that the path
  exists and still list the default folder as `Source::Fallback`.

### Q4: Epic Games Store

- **PoE2 is sold on Epic.** GGG's Early Access FAQ (Community_Team, 2024-11-21) lists "a standalone PC client, Steam
  and Epic Games Store clients" and lets a key bought for one client unlock another ([ggg-eafaq]). Epic's own
  store-content API returns `productName` "Path of Exile 2" and namespace `58a18be6bbba41a0bf52b014bb81d33b`
  ([epic-content]). PCGamingWiki lists Epic as a store ([pcgw]).
- **Manifests.** Epic: "On Windows, the Launcher typically stores manifest, dynamic, and temporary files in the
  system's ProgramData folder (for example, `%programdata%\Epic`)" ([epic-doc]). Legendary reads every `*.item` in
  `%PROGRAMDATA%\Epic\EpicGamesLauncher\Data\Manifests` ([legendary-lfs] L13–21, L49–60). Its field list ([legendary-model]
  L6–46) includes `AppName`, `DisplayName`, `InstallLocation`, `LaunchExecutable` (relative to `InstallLocation`,
  [jayd]), `CatalogNamespace`, `CatalogItemId`, `MainGameAppName`, `MandatoryAppFolderName` and
  `bIsIncompleteInstall`. Resolve `%PROGRAMDATA%` from the environment or `FOLDERID_ProgramData` (default
  `C:\ProgramData`, [ms-kfid]).
- **`LauncherInstalled.dat`.** A JSON object whose `InstallationList[]` entries carry at least `AppName` and
  `InstallLocation` ([bottles-egs] L31–36, L65–68). It is a second source if the manifest folder is missing.
- **Default Epic root** `C:\Program Files\Epic Games\` comes from Epic's help page ([epic-help]) **[1 source]**. The `.item` file's `InstallLocation` is
  authoritative anyway.
- **Identifiers** (Epic catalog via egdata):

| Catalog item | `AppName` (`releaseInfo.appId`) | `FolderName` | Source |
|---|---|---|---|
| `95f069d73abe4ba3a396a8399347f777` "Path of Exile 2" | `57d39134358849b1b765ee649ea6a6e9` | `PathOfExile2` | [egdata-item] |
| `2e200f9f0405461eb948b13f37f9897d` "Path of Exile 2 Free Weekend" | `54c5e100caf049bf88068afe4c02de21` | `POE2FreeWeekend` | [egdata-fw] |

  The sandbox has `displayName` "Path of Exile 2" and the internal `codeName` "Quesito" ([egdata-sandbox]).
- Synthetic `.item` example (real IDs, synthetic path):

```json
{ "AppName": "57d39134358849b1b765ee649ea6a6e9", "DisplayName": "Path of Exile 2",
  "CatalogNamespace": "58a18be6bbba41a0bf52b014bb81d33b", "CatalogItemId": "95f069d73abe4ba3a396a8399347f777",
  "InstallLocation": "D:\\Epic Games\\PathOfExile2", "bIsIncompleteInstall": false }
```

- **Implication.** Match on `CatalogNamespace`; the namespace comes from Epic's own API. Fall back to a
  `DisplayName` that starts with `Path of Exile 2`, which also covers the Free Weekend build. Keep a match with
  `bIsIncompleteInstall: true` and add a note to it.

### Q5: the Documents folder

- **API.** `SHGetKnownFolderPath(rfid, dwFlags, hToken, *ppszPath)`. The caller frees the result with
  `CoTaskMemFree` "whether SHGetKnownFolderPath succeeds or not". The path has no trailing backslash.
  `SHGetFolderPath` "is now simply a wrapper" for it ([ms-shgkfp]). `FOLDERID_Documents` =
  `{FDD39AD0-238F-46AF-ADB4-6C85480369C7}`, default `%USERPROFILE%\Documents`, CSIDL `CSIDL_MYDOCUMENTS` /
  `CSIDL_PERSONAL` ([ms-kfid]).
- **Flags.** Without `KF_FLAG_DEFAULT_PATH`, the call returns the "current—and possibly redirected—path". Without
  `KF_FLAG_DONT_VERIFY`, it checks that the folder exists and "returns a failure code" if it is "absent or
  inaccessible" ([ms-kfflag]). `KF_FLAG_DONT_VERIFY` therefore returns a redirected path even when its target has
  gone, so `poe2 doctor` can show it with `exists: false`.
- **Registry mirror.** `User Shell Folders` → `Personal` is REG_EXPAND_SZ (Microsoft's archived default is
  `%USERPROFILE%\My Documents`) ([ms-personal]). Raymond Chen asks programs not to rely on either the "Shell Folders"
  or the "User Shell Folders" key and to call the shell function instead ([chen]). Do not read the registry.
- **OneDrive.** Known Folder Move redirects "Desktop, Documents, Pictures, Screenshots, and Camera Roll" to OneDrive
  ([ms-kfm]). Player reports show PoE2 follows the redirection. Players with settings that would not save fixed it by keeping the OneDrive
  "My Games" folder on the device ([forum-onedrive]). In another case a stale OneDrive value in `User Shell Folders`
  broke filters until it was reset to `%USERPROFILE%\Documents` ([forum-filter]). `%USERPROFILE%\Documents`, which
  guides use ([neversink], [pcgw]), is therefore only a fallback.
- **Crates that wrap it** (crates.io, 2026-10-23, [cr-api]):

| Crate | Latest | Windows deps | Notes |
|---|---|---|---|
| `dirs` | 7.0.0 (2026-09-05), 13.8 KB | `dirs-sys` 0.5.0 → `option-ext`, `windows-sys` ≥0.59 | Calls `SHGetKnownFolderPath(FOLDERID_Documents, 0, …)` ([dirs-sys]). Flags fixed at 0, so the call fails when the folder is missing. It also brings XDG logic on Linux that we don't need. Repo moved to Codeberg. |
| `known-folders` | 1.4.2 (2026-03-17), 16.2 KB, MSRV 1.60 | `windows-sys` 0.59–0.61 | Safe wrapper, Windows-only, fixed to `KF_FLAG_DEFAULT` ([kf-src]). |
| `windows-sys` | 0.61.2 (2025-10-06), 2.5 MB download, compiled per feature, MSRV 1.71 | `windows-link` 0.2.1 | Microsoft. Has `SHGetKnownFolderPath`, `FOLDERID_Documents`, `KF_FLAG_DONT_VERIFY` and `CoTaskMemFree` (checked in the 0.61.2 source). |

### Q6: the PoE2 user directory

`<Documents>\My Games\Path of Exile 2\`, with Documents from the known-folder API:
- `poe2_production_Config.ini` is the game's configuration file ([pcgw], [exileui-cfg] L246–252). Players edit
  key bindings in it ([forum-cfg]).
- `*.filter` item filters sit in the folder itself, "directly where ingame folder button opens - not a subfolder"
  ([neversink] L47–50). A player says filters "belong in the Path of Exile 2 folder", not in `OnlineFilters`
  ([forum-filter]). Exile-UI reads filters and the config from the same folder ([exileui-cfg]).
- `OnlineFilters\` holds filters synced from the account or website. Users are told not to put files there
  ([neversink], [forum-filter]). Exile-UI reads that folder ([exileui-lf] L28).
- `BuildPlanner\` holds `*.build` files (all from [ggg-docs], "Build Planner (PoE2 only)"):
  - "To activate the Build Planner, at least one valid `*.build` file should be present in the build directory."
  - "The game expects `*.build` files to be written in JSON format containing a single Build object."
  - "Our automated File Watcher system has been designed to detect changes in the BuildPlanner directory."
  - Windows default: `C:/Users/<user>/Documents/My Games/Path of Exile 2/BuildPlanner` (GGG's spelling, forward
    slashes). Builds subscribed on pathofexile2.com "will automatically be loaded by the game". Editing builds in
    the game "is currently not supported".
  - `Build`, Version 1 (Experimental): `name` string (required); `author`, `link`, `description`, `ascendancy`
    ?string; `passives` ?array of (string or `BuildPassive{id, level_interval, weapon_set 0–2, additional_text}`);
    `skills` ?array of (string or `BuildSkill{id = BaseItemTypes id, level_interval, additional_text,
    support_skills: string or BuildSupport}`); `inventory_slots` ?array of
    `BuildInventorySlot{inventory_id, slot_x, slot_y, level_interval, unique_name, additional_text}`.
    `additional_text` accepts markup such as `<b>{…}`.

### Q7: registry crates and the recommendation

| Crate | Latest (date), size, MSRV | Deps | Verdict |
|---|---|---|---|
| `windows-registry` | 0.100.0 (2026-09-03), 13.8 KB, MSRV **1.95**; 0.6.1 (2025-10-06), MSRV 1.82 | 0.100: `windows-link`/`-result`/`-strings` 0.100. 0.6.1: `windows-link` 0.2.1, `windows-result` 0.4.1, `windows-strings` 0.5.1 | Microsoft, safe. `get_string` accepts REG_SZ and REG_EXPAND_SZ. `get_hstring().to_os_string()` keeps paths lossless. 0.100 adds `wow64_32()`; 0.6.1 only has `access(u32)`. 0.100 with `windows-sys` 0.61 builds two `windows-link` versions. |
| `winreg` | 0.56.0 (2026-03-14), 33.1 KB, MSRV 1.60, MIT only | `cfg-if`, `windows-sys` 0.59–0.61 (optional `chrono`, `serde`) | Mature (since 2015), one maintainer. Used by steamlocate ([steamlocate-win]). No advantage here. |
| `windows-sys` (raw `RegGetValueW`) | 0.61.2 (see Q5) | `windows-link` 0.2.1 | Has `RegGetValueW`, `RRF_RT_REG_SZ`, `RRF_SUBKEY_WOW6432KEY`, `HKEY_CURRENT_USER` (checked in the 0.61.2 source). |
| `steamlocate` 2.1.1 | MSRV 1.83 | `crc`, `keyvalues-parser`, `keyvalues-serde`, `serde`, `winreg` | Rejected. It reads the real disk and registry directly, so it cannot go through the `System` trait or `FakeSystem`, and its tree is heavy. |

**Recommendation.** Add one dependency, `[target.'cfg(windows)'.dependencies] windows-sys = "0.61"`, with features
`Win32_Foundation`, `Win32_UI_Shell`, `Win32_System_Com` and `Win32_System_Registry`. Put two small wrappers (about
40 lines of `unsafe`) in the real Windows `System` impl:
- `known_folder_documents()`: `SHGetKnownFolderPath` with `KF_FLAG_DONT_VERIFY`, then `CoTaskMemFree`.
- `reg_string(hive, subkey, value)`: `RegGetValueW` with `RRF_RT_REG_SZ`.

Why: `windows-sys` is the base of every alternative. It adds 2 crates on Windows and none on Linux, is maintained by
Microsoft, and needs MSRV 1.71, below the 1.85 floor that edition 2024 already sets. It is also the only option that
can pass `KF_FLAG_DONT_VERIFY`, which shows the OneDrive failure players report instead of hiding it. If hand-written
registry FFI is unwanted, add `windows-registry` 0.6. It shares `windows-link` 0.2 with `windows-sys` 0.61
and adds about 41 KB in 3 crates (0.6.1, 0.4.1, 0.5.1). Do not add `dirs`, `known-folders`, `winreg` or
`steamlocate`.

## Conflicting

1. **App ID `2694490` vs `2315204395`.** Steam's store, API and PICS, PCGamingWiki, NeverSink and one build guide
   use `2694490` ([store], [pics], [pcgw], [neversink], [guide-2694]). GGG's developer docs, copied by another guide,
   use `2315204395` ([ggg-docs], [guide-2315]). **Resolution:** `2694490` is the store app. `2315204395` is
   shortcut-style (high bit set; Valve SDK rule), so it is tied to whichever Steam install generated it. Never
   hardcode it (Q1).
2. **Standalone default folder: `Program Files (x86)` vs `Program Files`.** Players and the Lutris script say
   `(x86)` ([forum-dir], [forum-opt], [lutris-poe2]). PoE2MT lists `C:\Program Files\…` first and `(x86)` as "on some
   systems" ([poe2mt]); JuiceJournal probes both ([juice]). **Resolution:** list both as `Source::Fallback`
   after the registry candidate.
3. **Which `libraryfolders.vdf` to read.** Lutris and TraXile read `config\libraryfolders.vdf`
   ([lutris-cfg] L216, [traxile] L54). steamlocate and protontricks read `steamapps\libraryfolders.vdf`
   ([steamlocate-lib], [protontricks]). One player edits "the two files" ([forum-2copies]). I found no Valve
   statement on which copy is authoritative. **Resolution:** read both, merge without duplicates, then confirm
   each library by its `appmanifest_*.acf`.
4. **Documents written as a fixed path vs redirected.** NeverSink writes `%userprofile%\Documents\My Games\Path of
   Exile 2`, PCGamingWiki writes `%USERPROFILE%\Documents`, and GGG writes `C:/Users/<user>/Documents` ([neversink],
   [pcgw], [ggg-docs]). Known Folder Move can put Documents under OneDrive, and PoE2 follows it ([ms-kfm],
   [forum-onedrive]). **Resolution:** use the known-folder API. Keep `%USERPROFILE%\Documents` only as a fallback.
5. **Prefix Documents folder name on Linux.** NeverSink uses `users/steamuser/My
   Documents/…`, GGG uses `users/steamuser/Documents/…` ([neversink], [ggg-docs]). This is out of Windows scope; see [linux-paths.md](linux-paths.md).
6. **`OnlineFilters` vs `onlinefilters`.** Exile-UI and a forum post write `OnlineFilters`; NeverSink writes
   `onlinefilters` ([exileui-lf], [forum-filter], [neversink]). Windows ignores case; Linux file systems, where a
   reader sees Proton prefixes, do not. Match case-insensitively.

## Unknown

- **Exact origin of `2315204395`.** I hashed 1,584 plausible installer path and name pairs with
  `crc32(exe+name) | 0x80000000`. None matched. Heroic also adds shortcuts through the Steam client, which returns
  an ID ([heroic-add]). Only the high-bit rule is confirmed.
- **Standalone Uninstall entry** (hive, key name, `DisplayName`, `InstallLocation`). I found no source, and winget
  has no GGG manifest ([winget-g]). The "Options" button on the licence page looks like a WiX bootstrapper, but that
  is a guess.
  Check on a real machine: `reg query HKCU\Software\Microsoft\Windows\CurrentVersion\Uninstall /s /f "Path of Exile 2"`
  and the same under `HKLM\…\WOW6432Node\…`.
- **Who writes `GrindingGearGames\Path of Exile 2\InstallLocation`** (installer or client), whether the value has a
  trailing `\`, and whether a `Wow6432Node` copy exists for PoE2.
- **Is `SteamPath` always lower-case?** deadloader leaves case to comparison ([deadloader]). Compare
  case-insensitively whatever the answer.
- **ACF `StateFlags` meaning** (4 is common in fixtures). Valve does not document it. Do not filter on it; check
  that the folder exists instead.
- **Epic:** PoE2's `LaunchExecutable` name. Whether `DisplayName` is ever localised. Whether a Free Weekend install
  becomes the main `AppName` after purchase. The full `LauncherInstalled.dat` schema. The value
  `HKLM\SOFTWARE\WOW6432Node\Epic Games\EpicGamesLauncher\AppDataPath` (seen in one forum post only). The two
  `AppName`s rest on egdata alone; check them against a real `.item`.
- **User dir:** where website-subscribed builds are cached. Whether the game creates `BuildPlanner\` itself. Any
  other contents, such as screenshots. Whether the active filter is the `item_filter` key in
  the config's `[UI]` section (Exile-UI reads it, [exileui-lf] L27) **[1 source]**.
- **Does PoE2 call the known-folder API or read `User Shell Folders`?** The forum fix ([forum-filter]) fits either,
  because the shell keeps the real folder locations in that key ([chen]).
- **`windows-sys` 0.100** is not on crates.io as of 2026-10-23, but the windows-rs master README already shows
  `version = "0.100"` with header-style feature names ([wsys-readme]). Expect a small feature rename when it ships.

[store]: https://store.steampowered.com/app/2694490/Path_of_Exile_2/
[appdetails]: https://store.steampowered.com/api/appdetails?appids=2694490
[pics]: https://api.steamcmd.net/v1/info/2694490
[pcgw]: https://www.pcgamingwiki.com/wiki/Path_of_Exile_2
[sdk-gameid]: https://github.com/ValveSoftware/source-sdk-2013/blob/0759e2e8e179d5352d81d0d4aaded72c1704b7a9/src/public/steam/steamclientpublic.h#L1057-L1085
[sdk-scripts]: https://partner.steamgames.com/doc/sdk/installscripts
[heroic-id]: https://github.com/Heroic-Games-Launcher/HeroicGamesLauncher/blob/509a34feea8541b73c928e289f2cf262d432db4a/src/backend/shortcuts/nonesteamgame/steamhelper.ts#L199-L223
[heroic-add]: https://github.com/Heroic-Games-Launcher/HeroicGamesLauncher/blob/509a34feea8541b73c928e289f2cf262d432db4a/src/backend/shortcuts/nonesteamgame/nonesteamgame.ts#L569-L594
[winget-g]: https://github.com/microsoft/winget-pkgs/tree/master/manifests/g
[forum-sample]: https://steamcommunity.com/discussions/forum/1/3395163747110198261
[deadloader]: https://github.com/horexdev/deadloader/pull/4
[vibeshine]: https://github.com/Nonary/vibeshine/issues/314
[steambrew]: https://docs.steambrew.app/users/guides/finding-steam
[steamlocate-win]: https://github.com/WilliamVenner/steamlocate-rs/blob/d740b423aee82e8e742595d0cdb4edcc91c24216/src/locate/windows.rs#L15-L28
[steamlocate-lib]: https://github.com/WilliamVenner/steamlocate-rs/blob/d740b423aee82e8e742595d0cdb4edcc91c24216/src/library.rs
[steamlocate-acf]: https://github.com/WilliamVenner/steamlocate-rs/blob/d740b423aee82e8e742595d0cdb4edcc91c24216/tests/assets/appmanifest_230410.acf
[steamlocate-marker]: https://github.com/WilliamVenner/steamlocate-rs/blob/d740b423aee82e8e742595d0cdb4edcc91c24216/tests/assets/libraryfolder.vdf
[protontricks]: https://github.com/Matoking/protontricks/blob/23f19063999bb016fc8e1be048b76e4c5e4c78dc/tests/conftest.py#L467-L480
[lutris-cfg]: https://github.com/lutris/lutris/blob/1f32fe0e46e2a5d92fad9db2846c7cb7becd20c3/lutris/util/steam/config.py#L213-L285
[traxile]: https://github.com/dermow/TraXile/blob/f86a83f15c72db9d89446e7f11781e44bef1d5c8/TraXile/Trx_PoeLogFileDetector.cs#L12-L58
[forum-vdf]: https://steamcommunity.com/discussions/forum/1/3421063428083444843
[forum-2copies]: https://steamcommunity.com/discussions/forum/1/4333105589464423275
[vdc-kv]: https://developer.valvesoftware.com/wiki/KeyValues
[ms-naming]: https://learn.microsoft.com/en-us/windows/win32/fileio/naming-a-file
[ggg-docs]: https://www.pathofexile.com/developer/docs/game#buildplanner
[ggg-eafaq]: https://www.pathofexile.com/forum/view-thread/3587981
[forum-opt]: https://www.pathofexile.com/forum/view-thread/3593206
[forum-dir]: https://www.pathofexile.com/forum/view-thread/3593077
[forum-reg]: https://www.pathofexile.com/forum/view-thread/3593313
[forum-cfg]: https://www.pathofexile.com/forum/view-thread/3593244
[forum-onedrive]: https://pathofexile.com/forum/view-thread/3599901
[forum-filter]: https://www.pathofexile.com/forum/view-thread/3609880
[lutris-poe2]: https://lutris.net/games/install/39535/view
[guide-2315]: https://brickerp.github.io/poe2-build-lab/guides/import-build-files.html
[guide-2694]: https://poe2buildfile.com/poe2-build-planner-import/
[epic-content]: https://store-content-ipv4.ak.epicgames.com/api/en-US/content/products/path-of-exile-2
[egdata-sandbox]: https://api.egdata.app/sandboxes/58a18be6bbba41a0bf52b014bb81d33b
[egdata-item]: https://api.egdata.app/items/95f069d73abe4ba3a396a8399347f777
[egdata-fw]: https://api.egdata.app/items/2e200f9f0405461eb948b13f37f9897d
[epic-doc]: https://dev.epicgames.com/documentation/unreal-engine/multiple-launcher-unreal-engine-installs
[epic-help]: https://www.epicgames.com/help/c-Category_EpicGamesStore/c-EpicGamesStore_LauncherSupport/epic-games-a000089803
[legendary-lfs]: https://github.com/derrod/legendary/blob/42f6bdeadde3a9526dc8eb713763476999ac217b/legendary/lfs/egl.py#L13-L60
[legendary-model]: https://github.com/derrod/legendary/blob/42f6bdeadde3a9526dc8eb713763476999ac217b/legendary/models/egl.py#L6-L46
[bottles-egs]: https://github.com/bottlesdevs/Bottles/blob/cfd96650a929fcafef1fc2b3e6cd23e5b144a44d/bottles/backend/managers/epicgamesstore.py#L31-L68
[jayd]: https://jayd.ml/games/2020/05/16/epic-games-store-steam-libraries.html
[ms-kfid]: https://learn.microsoft.com/en-us/windows/win32/shell/knownfolderid
[ms-shgkfp]: https://learn.microsoft.com/en-us/windows/win32/api/shlobj_core/nf-shlobj_core-shgetknownfolderpath
[ms-kfflag]: https://learn.microsoft.com/en-us/windows/win32/api/shlobj_core/ne-shlobj_core-known_folder_flag
[ms-kfm]: https://learn.microsoft.com/en-us/sharepoint/redirect-known-folders
[ms-personal]: https://learn.microsoft.com/en-us/previous-versions/windows/it-pro/windows-2000-server/cc962616(v=technet.10)
[chen]: https://devblogs.microsoft.com/oldnewthing/20031103-00/?p=41973
[neversink]: https://github.com/NeverSinkDev/NeverSink-Filter-for-PoE2/blob/fa17bd462dca1630b0b90829fecdfc2894e4688d/README.md#L47-L50
[exileui-cfg]: https://github.com/Lailloken/Exile-UI/blob/67d1a3637e97b791072129f0cf890e22f385b237/Exile%20UI.ahk#L246-L252
[exileui-lf]: https://github.com/Lailloken/Exile-UI/blob/67d1a3637e97b791072129f0cf890e22f385b237/modules/lootfilter.ahk#L26-L28
[poe2mt]: https://github.com/igodspy/PoE2MT/blob/b4b4c2bff43c51f4a0ce21cb41af47f4bd2c0241/README.md
[splitstone]: https://github.com/noman-omen/poe2-splitstone/blob/f3c66aa6a5390413505fb461b7d4d51e95d74c67/helper/splitstone_helper.py#L44-L45
[juice]: https://github.com/JuiceJournal/JuiceJournal/blob/c33050ba4db7b01322f850c3c3994d9cd5f3ce11/desktop/src/modules/gameDetector.js#L44-L49
[cr-api]: https://crates.io/api/v1/crates/windows-registry
[dirs-sys]: https://docs.rs/crate/dirs-sys/0.5.0/source/src/lib.rs
[kf-src]: https://docs.rs/crate/known-folders/1.4.2/source/src/win.rs
[wsys-readme]: https://github.com/microsoft/windows-rs/blob/master/crates/libs/sys/readme.md
