# PoE2 Client.txt: where it lives and what its lines look like

Researched 2026-10-23 for the `log` module and the locator. Sources are public only:
twelve open-source parsers cloned read-only (commits in the source key), one file fetched from GitHub, and
GGG's developer docs. No real Client.txt was opened, and every example line here is synthetic.
"PoE2 source" means a project that targets PoE2 or both games. **(1)** marks a claim that has only one source.

## Confirmed

### Summary

`P` stands for the line prefix `<date> <time> <ms> <hex> [<LEVEL> Client <pid>]` (see "Line prefix" below).
Every synthetic example uses the prefix `2026/01/02 03:04:05 123456789 1a2b3c4d [INFO Client 4242]` (or `DEBUG`), shortened to `…` in the table.

| Event | Pattern | Synthetic example | Sources | Status |
|---|---|---|---|---|
| Log opened | `<date> <time> ***** LOG FILE OPENING *****` (whole line; no other fields) | `2026/01/02 03:04:05 ***** LOG FILE OPENING *****` | [JJ][jj-fix], [MW][mw-open], [EU][eu-parse]; PoE1: [TX][tx-map] | CONFIRMED |
| Area generated | `P` (DEBUG) `Generating level <lvl> area "<area_id>" with seed <seed>` | `… [DEBUG Client 4242] Generating level 2 area "G1_2" with seed 3000000001` | [JJ][jj-fix], [SG][sg-parse], [SS][ss-parser], [MT][mt-app], [MW][mw-gen], [EU][eu-parse], [EL][el-main], [QU][qu-tags] | CONFIRMED |
| Scene set (area name) | `P` (INFO) `[SCENE] Set Source [<area_name>]`; `(null)` and `(unknown)` are sentinels | `… [INFO Client 4242] [SCENE] Set Source [Clearfell]` | [JJ][jj-fix], [SG][sg-core], [MT][mt-app], [PN][pn-watch], [QU][qu-tags] | CONFIRMED |
| Zone entered | `P` (INFO) `: You have entered <area_name>.` | `… [INFO Client 4242] : You have entered Clearfell.` | [SS][ss-tests], [JJ][jj-entered], [PN][pn-watch]; PoE1: [PLM][plm-sys], [TX][tx-map] | CONFIRMED (may be absent; see Conflicting) |
| Level-up | `P` (INFO) `: <name> (<class>) is now level <n>` (no final period) | `… [INFO Client 4242] : ExileOne (Smith of Kitava) is now level 12` | [SS][ss-lvl], [JJ][jj-model], [SG][sg-parse], [MW][mw-slain-level], [QU][qu-level], [PN][pn-watch], [EU][eu-lang]; PoE1: [PLM][plm-sys] | CONFIRMED |
| Character slain | `P` (INFO) `: <name> has been slain.` | `… [INFO Client 4242] : ExileOne has been slain.` | [JJ][jj-fix], [SG][sg-parse], [MT][mt-app], [MW][mw-slain-level], [EU][eu-lang]; PoE1: [PLM][plm-sys], [TX][tx-map] | CONFIRMED |
| Whisper received | `P` (INFO) `@From <name>: <text>` | `… [INFO Client 4242] @From TraderJoe: Hi, is the bow still for sale?` | [MW][mw-whisper], [JJ][jj-model], [SS][ss-noise]; PoE1: [PLM][plm-chat] | CONFIRMED |
| Whisper sent | `P` (INFO) `@To <name>: <text>` | `… [INFO Client 4242] @To TraderJoe: Yes, sending an invite.` | [MW][mw-whisper], [JJ][jj-model] (comment only); PoE1: [PLM][plm-chat] | CONFIRMED (thin: no PoE2 fixture) |
| Whisper guild tag | `@From <<tag>> <name>: <text>` (also `@To`) | `… [INFO Client 4242] @From <GLD> TraderJoe: Hi` | PoE1 only: [PLM][plm-chat] | UNCONFIRMED |
| AFK on | `P` (INFO) `: AFK mode is now ON` + suffix; PoE1 suffix is `. Autoreply "<text>"` | `… [INFO Client 4242] : AFK mode is now ON. Autoreply "Back in 5"` (suffix as in PoE1) | [SG][sg-parse], [SS][ss-noise]; PoE1: [PLM][plm-chat] | CONFIRMED (stem only) |
| AFK off | `P` (INFO) `: AFK mode is now OFF` + optional `.` | `… [INFO Client 4242] : AFK mode is now OFF.` | [SG][sg-parse]; PoE1: [PLM][plm-chat] | CONFIRMED (stem only) |
| DND on / off | `P` (INFO) `: DND mode is now ON. Autoreply "<text>"` / `: DND mode is now OFF` | `… [INFO Client 4242] : DND mode is now ON. Autoreply "Busy"` | PoE1 only: [PLM][plm-chat] | UNCONFIRMED |

### Where Client.txt lives

**Verdict:** the file is `logs/Client.txt` inside the game's **install directory**, next to the executable.
No source puts it under `Documents`. Exile-UI builds the path from the running process's executable folder
([EU][eu-loc]), and exile-log-api builds it from the process's working directory ([EL][el-main]).

| Install | Path | Sources | Status |
|---|---|---|---|
| Steam, Windows | `<library>\steamapps\common\Path of Exile 2\logs\Client.txt` | [SS][ss-paths], [JJ][jj-paths], [PN][pn-paths], [SG][sg-paths], [MT][mt-readme], [QU][qu-path], [EE][ee-watch], [SB][sb] | CONFIRMED |
| Steam + Proton, Linux | `<library>/steamapps/common/Path of Exile 2/logs/Client.txt`, for example under `~/.local/share/Steam` or `~/.steam/steam`. This is the native install path, not inside `compatdata`. | [SS][ss-helper-paths], [EE][ee-watch] | CONFIRMED |
| Standalone, Windows | `C:\Program Files (x86)\Grinding Gear Games\Path of Exile 2\logs\Client.txt`, or the same under `C:\Program Files\` | [EE][ee-watch], [SS][ss-paths], [PN][pn-paths], [SG][sg-paths], [JJ][jj-paths], [MT][mt-readme] | CONFIRMED |
| Standalone under Wine | `~/.wine/drive_c/Program Files (x86)/Grinding Gear Games/Path of Exile 2/logs/Client.txt` | [EE][ee-watch] **(1)** | UNCONFIRMED |
| Epic | `C:\Program Files\Epic Games\PathOfExile2\logs\Client.txt`. PoE2 is sold on Epic ([store page][epic]). | [PN][pn-paths] **(1)** | UNCONFIRMED |
| Kakao (Korea) | `<install>\logs\KakaoClient.txt`, for example `C:\Daum Games\Path of Exile2\logs\KakaoClient.txt` | [SG][sg-paths], [EU][eu-loc] | CONFIRMED |
| `LatestClient.txt` | `<install>\logs\LatestClient.txt`, next to Client.txt; see Unknown | [EU][eu-loc], [EL][el-main] (neither is PoE2-specific) | UNCONFIRMED for PoE2 |
| Documents | `Documents\My Games\Path of Exile 2\` holds `poe2_production_Config.ini` and `OnlineFilters`, not the log | [SG][sg-dev], [SB][sb] | CONFIRMED (no log there) |

No source reports numbered or rotated logs. `Client (old).txt` is a backup that Exile-UI creates, not a file the game writes ([EU][eu-backup]).
poe2-saegida checks several candidate paths and picks the most recently modified file ([SG][sg-paths]).

### Line prefix

```
<date>     <time>   <ms>      <hex>    [<LEVEL> <source> <pid>] <body>
2026/01/02 03:04:05 123456789 1a2b3c4d [INFO Client 4242] : You have entered Clearfell.
```

A single space separates every field. A grammar that fits all the PoE2 evidence (my own wording, not taken from any tool):
`^(\d{4}/\d{2}/\d{2}) (\d{2}:\d{2}:\d{2}) (\d+) ([0-9a-fA-F]+) \[([A-Z]+) (\S+) (\d+)\] (.*)$`.

| Field | Format | Meaning | Sources | Status |
|---|---|---|---|---|
| date | `YYYY/MM/DD`, zero-padded | Calendar date; written with `/` in the Korean and Japanese clients too | all; KO: [SG][sg-core], JA: [PN][pn-fixtures] | CONFIRMED |
| time | `hh:mm:ss`, 24-hour, no fractions, no zone | Wall-clock time. Parsers read it as local time: JJ compares it with the local `now`, SG uses naive datetimes. | [JJ][jj-ts], [SG][sg-ts] | format CONFIRMED; time zone UNCONFIRMED |
| `<ms>` | decimal integer | A millisecond counter. In the JJ 0.5.5e fixture it grows by 4,719 over 5 s and 48,672 over 49 s. Values up to about 9.5×10⁸ (SG) suggest time since OS boot. | [JJ][jj-fix], [SG][sg-core] | integer CONFIRMED; epoch unknown |
| `<hex>` | hex digits, variable width (7- and 8-digit values seen) | Opaque. Within one session, one message type always carries the same value. The value on `Generating level` lines differs in JJ fixtures dated May and October 2026, probably different builds. | [JJ][jj-fix], [JJ][jj-tests], [SS][ss-parser] | form CONFIRMED; meaning unknown |
| `<LEVEL>` | `INFO`, `DEBUG` | Log level. `Generating level` and `Got Instance Details` lines are DEBUG; chat, system messages and `[SCENE]` lines are INFO. | [JJ][jj-fix], [SG][sg-core], [MW][mw-gen] | CONFIRMED (only these two seen) |
| `<source>` | `Client` | The only value seen | all | CONFIRMED |
| `<pid>` | decimal; constant for one game run | Process id. SG names it so and uses a change of value to detect a game restart. | [SG][sg-dev] **(1)**, constant in [JJ][jj-fix] | UNCONFIRMED (name) |

Body shapes (what follows `] `):

| Body starts with | Kind | Synthetic body | Sources |
|---|---|---|---|
| `: ` | A system or game message. It is localised. A player cannot produce it, because chat lines always start with the sender. | `: ExileOne has been slain.` | [SS][ss-lvl], [SG][sg-parse], [JJ][jj-model] |
| `#`, `$`, `%`, `&`, `@From `, `@To `, or `<name>: ` | Chat: global, trade, party, guild, whisper received or sent, and local. NPC dialogue uses the local-chat shape too (`<npc>: <text>`). | `$TraderJoe: WTS bow, 5 ex` | [JJ][jj-model], [SG][sg-parse]; PoE1: [poeseed][ps-awk], [PLM][plm-chat] |
| anything else | An engine message, always in English, for example `Generating level …`, `[SCENE] Set Source […]`, `[WINDOW] Gained focus`, `[WINDOW] Lost focus`, `[STARTUP] Loading Start`, `[LOADING SCREEN] (<area_name>) Duration = <seconds>`, `Async connecting to <realm>.login.pathofexile2.com:<port>`, `Got Instance Details from login server`, `Connecting to instance server at <ip>:<port>`, `Closing game gracefully` | `Connecting to instance server at 192.0.2.10:6112` | [JJ][jj-fix], [QU][qu-tags], [MT][mt-app], [SG][sg-parse] |

Match each event on the start of the body, never by substring. In JJ's 0.5.5e fixture a trade-chat line contains
` : <name> (<class>) is now level <n>` ([JJ][jj-fix]). SS's tests reject look-alikes in local, global, whisper and party chat
(`<name>: …`, `#<name>: : …`, `@From <name>: …`, `%<name>: x] : …`), a DEBUG copy, and malformed names, classes and levels ([SS][ss-lvl], [JJ][jj-spoof]).
Real logs contain server IPs in `Connecting to instance server` lines (JJ redacts them), so fixtures must use 192.0.2.0/24.

### File properties

| Property | Finding | Sources | Status |
|---|---|---|---|
| Encoding | UTF-8. Korean and Japanese bodies decode as UTF-8 in tools built around those clients. | [SG][sg-i18n], [PN][pn-watch], [JJ][jj-utf8], [MT][mt-utf8], [EU][eu-tail], [EE][ee-watch] | CONFIRMED |
| Line endings | CRLF. SS tests a level-up line that ends in `\r`, and EU writes CRLF when it rewrites the file. Others split on `\r?\n`. | [SS][ss-lvl], [EU][eu-backup], [MT][mt-utf8], [SG][sg-parse] | CONFIRMED (accept LF too) |
| Growth | Append-only across sessions; each launch appends a `***** LOG FILE OPENING *****` line. No source reports the game rotating or truncating the file. | [JJ][jj-fix], [MW][mw-open], [EU][eu-help], [MT][mt-size] | CONFIRMED |
| Size | MT: "can grow for weeks and become very large". SG replays a 150 MB log in about 2 s. EU's log-health meter goes from green at 100 MB to red at 355 MB. | [MT][mt-size], [SG][sg-dev], [EU][eu-meter] | CONFIRMED (hundreds of MB occur) |

### Event details

**Area generated**: `[DEBUG Client <pid>] Generating level <lvl> area "<area_id>" with seed <seed>`
- `<lvl>` is the area level, in decimal. EU works out a map tier as `lvl − 64` for PoE2 ([EU][eu-parse]).
- `<area_id>` is the internal id, the same in every language: `G1_1`, `G1_town`, `G1_2`, `HideoutCanal`, `Map<Name>`. Case varies, so SS and SG lowercase it. EU strips a stray trailing `_` from two ids it calls "bugged PoE2 areaIDs" (`c_g2_9_2_`, `c_g3_16_`) **(1)**. Store the id as an opaque string.
- `<seed>` is decimal; fixtures include values above 2³¹, so parse it as u64. Hideouts log seed `1` ([JJ][jj-fix], [JJ][jj-tests]).
- The line is written as the instance is created. `[SCENE] Set Source [<area_name>]` follows ([JJ][jj-fix], [MT][mt-app]).
- PoE1 writes the same line: MW and EU parse both games with one rule. Localisation: never translated. SG uses one English regex for Korean and English clients, and its Korean docs quote the English line ([SG][sg-dev]).

**Scene set**: `[INFO Client <pid>] [SCENE] Set Source [<area_name>]`
- `(null)` appears while loading and `(unknown)` at the login and character-select screens ([JJ][jj-fix]). SG, MT and PN ignore both values, plus the empty one.
- The wrapper stays in English, but `<area_name>` is localised: Korean in [SG][sg-core], Japanese in [PN][pn-fixtures]. SG uses that name to detect the client language ([SG][sg-i18n]).
- The full load sequence in the 0.5.5e fixture is: `Set Source [(null)]`, then `Got Instance Details from login server`, `Connecting to instance server at …`, `Generating level …`, and `Set Source [<area_name>]` ([JJ][jj-fix]).

**Zone entered**: `[INFO Client <pid>] : You have entered <area_name>.`
- `<area_name>` is the display name, and the line ends with `.`. It usually follows `Generating level`; SS pairs the two lines within 5 s ([SS][ss-parser]).
- Localised. Japanese: `あなたは<area_name>に入場しました。` ([PN][pn-watch]; the game version is unclear). German (a PoE1-era example): `Ihr habt '<area_name>' betreten.` ([EU][eu-lang]) **(1)**. No sources give a Korean sample.

**Level-up**: `[INFO Client <pid>] : <name> (<class>) is now level <n>`
- `<class>` is the base class or the ascendancy, and can be several words. SS tests seven, including `Smith of Kitava`, `Acolyte of Chayula` and `Blood Mage` ([SS][ss-lvl], [SS][ss-changelog]).
- There is no final period: SS, MW and JJ anchor `<n>` at the end of the line, and SS allows a trailing `\r`. SS accepts levels 1 to 100 only.
- The line also appears when party members level up ([SS][ss-readme], [JJ][jj-model], [SG][sg-dev]), so the event carries `name` and the consumer decides which character it is.
- Localised, and the word order changes. Korean: `: <name>(<class>) 님이 <n>레벨이 되었습니다.`, with no space before `(` and a localised class name ([SG][sg-parse]) **(1)**. Japanese: `<name> (<class>)はレベル<n>になりました` ([PN][pn-watch]). EU notes that Japanese uses different parentheses and a different full stop ([EU][eu-lang]).

**Character slain**: `[INFO Client <pid>] : <name> has been slain.`
- The line also appears for party members ([JJ][jj-fix]). Korean: `: <name> 님이 사망했습니다.` ([SG][sg-parse]) **(1)**.

**Whispers**: `@From <name>: <text>` (received) and `@To <name>: <text>` (sent), straight after `] ` with no `: ` first.
- Split the line at the first `: ` after the name, because `<text>` can itself contain `: `. Tools match names with `\S+` or `[^:]+`. Kakao names can be Hangul ([SG][sg-parse]), so don't limit names to ASCII.
- PoE1 allows a guild tag before the name: `@From <<tag>> <name>: …` ([PLM][plm-chat]). A parser can accept an optional `<…> ` group at no cost.

**AFK**: the PoE2 sources match only the stem `: AFK mode is now ON` or `OFF` ([SG][sg-parse]). SS's noise fixture is `: AFK mode is now ON.` ([SS][ss-noise]).
- In PoE1 the ON line adds `. Autoreply "<text>"` ([PLM][plm-chat]). The PoE2 Korean ON line includes the autoreply text too: `: 자리 비움 모드를 설정했습니다. "<text>"을(를) 자동으로 답신합니다.` The OFF line is `: 자리 비움 모드를 해제했습니다.` ([SG][sg-parse], [SG][sg-afk]) **(1)**.
- A tolerant match: `^: (AFK|DND) mode is now (ON|OFF)`, then an optional `. Autoreply "<text>"`, then an optional `.`.

### Localisation: what is translated

- **Always English:** the prefix, the `[LEVEL Client pid]` tag, `***** LOG FILE OPENING *****`, `Generating level … area "<id>" with seed …`, the `[SCENE] Set Source [` wrapper, the connection lines, and the wrapper in `Successfully allocated passive skill id: <id>, name: <localised name>` ([SG][sg-parse], [PN][pn-fixtures]).
- **Translated:** every `: ` system message (zone entered, level-up, slain, AFK), class names, and area display names (in `[SCENE]` and in zone entered). Sentence order and punctuation change too ([SG][sg-parse], [PN][pn-watch], [EU][eu-lang]).
- The file name doesn't tell you the language: the Kakao client can be switched to English ([SG][sg-i18n]).
- GGG's own documentation says only this: "Reading the game's log files is okay as long as the user is aware of what you are doing with that data." ([GGG developer docs][ggg-dev]). No GGG page documents the format or the location, and I found no patch note that mentions Client.txt.

### Source key (local read-only clones; permalinks pinned to these commits)

| Key | Project | Commit | Games |
|---|---|---|---|
| JJ | [JuiceJournal/JuiceJournal](https://github.com/JuiceJournal/JuiceJournal) | `c33050ba4db7b01322f850c3c3994d9cd5f3ce11` | PoE1+2; its fixture is a "PoE2 0.5.5e" excerpt with made-up names |
| SS | [noman-omen/poe2-splitstone](https://github.com/noman-omen/poe2-splitstone) | `f3c66aa6a5390413505fb461b7d4d51e95d74c67` | PoE2 |
| SG | [ktu0216/poe2-saegida](https://github.com/ktu0216/poe2-saegida) | `6796ddf3db2cb8040ae4a73bbbf5507bec4c09be` | PoE2 (Korean-first; English lines from a 0.2.2 playthrough, [manual][sg-manual]) |
| MT | [igodspy/PoE2MT](https://github.com/igodspy/PoE2MT) | `b4b4c2bff43c51f4a0ce21cb41af47f4bd2c0241` | PoE2 |
| MW | [ugimser/mapwatch](https://github.com/ugimser/mapwatch) | `f3b5b9c3bae430d1311965755c026109b582b0c4` | PoE1+2 ([README][mw-readme]) |
| PN | [buri34/poenavi](https://github.com/buri34/poenavi) | `5a485be8469a97c7f96f0bfd1b00794f6ec6436f` | PoE1+2 (Japanese) |
| QU | [Vyary/qutm](https://github.com/Vyary/qutm) | `de642de860cee9284aba1ded9cdd0920b30555df` | PoE2 |
| EU | [Lailloken/Exile-UI](https://github.com/Lailloken/Exile-UI) | `67d1a3637e97b791072129f0cf890e22f385b237` | PoE1+2 |
| EL | [HeartofPhos/exile-log-api](https://github.com/HeartofPhos/exile-log-api) | `a0e75c0c8ef4c55cc29b29f5796d365be88157ce` | not stated |
| EE | [Kvan7/Exiled-Exchange-2](https://github.com/Kvan7/Exiled-Exchange-2) (one file fetched, not cloned) | `cca30662bf31eaf38bd711e2ec1a6b899a06c40e` | PoE2 |
| PLM | [klayveR/poe-log-monitor](https://github.com/klayveR/poe-log-monitor) | `d61aa3334a0f6b1f1ca9cef8164e058d2b9b4349` | PoE1 (2020) |
| TX | [dermow/TraXile](https://github.com/dermow/TraXile) | `f86a83f15c72db9d89446e7f11781e44bef1d5c8` | PoE1 |
| poeseed | [TwentyFiveEx/poeseed](https://github.com/TwentyFiveEx/poeseed) | `61366ed5fbf93727136779427268c7435c195147` | PoE1 (2022) |
| SB | [switchbladegaming.com guide](https://www.switchbladegaming.com/path-of-exile-2/save-file-location-4/) | n/a | secondary source, PoE2 0.2.x |

## Conflicting

1. **Whether "You have entered" is written at all.** SS's auto-split depends on it ([SS][ss-parser]), and JJ and PN parse it.
   But PN's README says the Local chat channel must be enabled, or some zone changes go undetected ([PN][pn-readme]) **(1)**.
   JJ's measured 0.5.5e excerpt shows a hideout load (`Generating level` followed by `Set Source [<area_name>]`) with no such line ([JJ][jj-fix]).
   EU keeps `"You have entered"` only in comments ([EU][eu-lang]). *Take:* emit the zone event from `[SCENE] Set Source` as well, and never require both lines.
2. **The level tag on `[SCENE]` lines.** It is INFO in JJ's 0.5.5e fixture and in SG's fixtures, but DEBUG in one PN fixture ([PN][pn-fixtures]).
   PN's fixtures also drop the numeric fields and the pid, so they are hand-trimmed. *Take:* don't key events on the level.
3. **The tag on `Generating level` lines.** In measured fixtures it is DEBUG with no `: ` (JJ 0.5.5e, SG, SS), and MW's regex requires DEBUG.
   JJ's own older unit tests write `[INFO Client 123] : Generating level …` ([JJ][jj-tests-info]). That test is synthetic.
   *Take:* match `Generating level` at the start of the body at any level, and tolerate a leading `: `.
4. **`: ` on localised system messages.** SG's Korean fixtures keep `] : `. PN's Japanese fixtures have none (`[INFO Client] あなたは…`) ([PN][pn-fixtures]), but they are trimmed (see 2).
   This is unresolved, and it doesn't matter for an English-only v1.
5. **The trailing period on zone entered.** PN, PLM and JJ require a `.` (`([^.]+)\.`); SS makes it optional (`\.?\s*$`).
   *Take:* strip one final `.`; don't use `[^.]+`, which truncates any name that contains a period.
6. **The character set of names and classes.** SS limits names to `[A-Za-z0-9_]{3,23}` and classes to ASCII words; SG and JJ use `\S+`.
   QU's `\((\w*)\)` can't match multi-word ascendancies that SS tests ([QU][qu-level] vs [SS][ss-lvl]). *Take:* `\S+` for names; anything up to `) ` for the class.
7. **Which file to tail.** EU tails `LatestClient.txt` when it exists ([EU][eu-tail]); every PoE2-only tool reads `Client.txt`. *Take:* v1 reads Client.txt; LatestClient.txt becomes a ticket.
8. **The standalone default folder.** EE and PN check only `Program Files (x86)` ([EE][ee-watch], [PN][pn-paths]). MT calls `Program Files` the common one and `(x86)` the variant ([MT][mt-readme]). SS, SG and JJ check both. *Take:* check both.

## Unknown

1. **The meaning of `<ms>` and `<hex>`.** `<ms>` behaves like a millisecond tick (its epoch is unknown, and it may wrap at 2³² if it comes from a 32-bit tick).
   `<hex>` looks like a per-call-site id that changes between builds. Keep both as opaque values (u64 and string).
2. **The meaning of `<pid>`.** It is the process id per SG only.
3. **Other levels and sources.** No PoE2 source shows `WARN`, `CRIT`, `ERROR` or any source other than `Client`. Accept any `[A-Z]+` level and any `\S+` source.
4. **A BOM.** Nothing shows the game writing one. But EU's backup recreates Client.txt in AutoHotkey "UTF-8" mode ([EU][eu-backup]), and per the [AHK docs][ahk] that mode writes a BOM when it creates a file. Strip a leading U+FEFF.
5. **Blank and multi-line records.** EU, MT and EE skip empty lines, but no source says where blank lines occur. No source shows a record that spans several lines.
   PN feeds unprefixed lines to its tests ([PN][pn-fixtures]). Unprefixed lines must become `Event::Unknown`.
6. **Truncation and rotation.** No source reports the game rotating or truncating the file. According to EU, the file "can only be modified after closing the game" (Windows) **(1)** ([EU][eu-help]), and the game creates a new file at launch if it is missing **(1)** ([EU][eu-backup]).
   It is unknown what the game does if the file is truncated while it runs (possible on Linux, where Windows sharing locks don't apply). JJ treats a shrink as rotation and restarts from 0 ([JJ][jj-shrink]).
7. **What `LatestClient.txt` is.** It sits in `logs/` next to Client.txt ([EU][eu-loc], [EL][el-main]). EU reads a `settings directory: <path>` line from it to find the config folder **(1)**.
   EU closes it when the game exits and reopens it after the game restarts ([EU][eu-tail]), and it shows its Client.txt size meter only when LatestClient.txt is absent ([EU][eu-meter]).
   Both suggest a small file holding only the current session. That is an inference.
   Whether PoE2 writes this file, and when it was introduced, has no source.
8. **Other installs.** The Epic path **(1)** and the Wine standalone path **(1)** each have one source. No PoE2 source covers Flatpak Steam, Lutris, Bottles or Heroic prefixes, or the Tencent (China) client.
9. **Other PoE2 forms.** No source shows DND lines, guild tags in whispers, or the English AFK-ON autoreply suffix in PoE2. Whether `@From`/`@To` are translated is unknown. EU lists `: <name> has committed suicide.` for "PoE1 and PoE2" **(1)** ([EU][eu-lang]).
10. **The time zone of timestamps.** Local time is assumed; no GGG statement covers it.
11. **Lines removed by patches.** EU notes that a log message (`current input mode = …`) "was removed at some point from PoE2 but is still present in PoE1" ([EU][eu-removed]).
    Any line can disappear in a patch, which is another reason to keep `Event::Unknown { raw }`.

[jj-fix]: https://github.com/JuiceJournal/JuiceJournal/blob/c33050ba4db7b01322f850c3c3994d9cd5f3ce11/desktop/tests/game-session-model.test.js#L6-L29
[jj-spoof]: https://github.com/JuiceJournal/JuiceJournal/blob/c33050ba4db7b01322f850c3c3994d9cd5f3ce11/desktop/tests/game-session-model.test.js#L64-L68
[jj-model]: https://github.com/JuiceJournal/JuiceJournal/blob/c33050ba4db7b01322f850c3c3994d9cd5f3ce11/desktop/src/modules/gameSessionModel.js#L9-L21
[jj-paths]: https://github.com/JuiceJournal/JuiceJournal/blob/c33050ba4db7b01322f850c3c3994d9cd5f3ce11/desktop/src/modules/gameDetector.js#L26-L54
[jj-utf8]: https://github.com/JuiceJournal/JuiceJournal/blob/c33050ba4db7b01322f850c3c3994d9cd5f3ce11/desktop/src/modules/logParser.js#L80
[jj-shrink]: https://github.com/JuiceJournal/JuiceJournal/blob/c33050ba4db7b01322f850c3c3994d9cd5f3ce11/desktop/src/modules/logParser.js#L149-L151
[jj-entered]: https://github.com/JuiceJournal/JuiceJournal/blob/c33050ba4db7b01322f850c3c3994d9cd5f3ce11/desktop/src/modules/logParser.js#L239
[jj-ts]: https://github.com/JuiceJournal/JuiceJournal/blob/c33050ba4db7b01322f850c3c3994d9cd5f3ce11/desktop/src/modules/logParser.js#L564-L580
[jj-tests]: https://github.com/JuiceJournal/JuiceJournal/blob/c33050ba4db7b01322f850c3c3994d9cd5f3ce11/desktop/tests/log-parser.test.js#L30-L32
[jj-tests-info]: https://github.com/JuiceJournal/JuiceJournal/blob/c33050ba4db7b01322f850c3c3994d9cd5f3ce11/desktop/tests/log-parser.test.js#L97
[ss-paths]: https://github.com/noman-omen/poe2-splitstone/blob/f3c66aa6a5390413505fb461b7d4d51e95d74c67/desktop/src/client-log.js#L11-L16
[ss-helper-paths]: https://github.com/noman-omen/poe2-splitstone/blob/f3c66aa6a5390413505fb461b7d4d51e95d74c67/helper/splitstone_helper.py#L42-L47
[ss-parser]: https://github.com/noman-omen/poe2-splitstone/blob/f3c66aa6a5390413505fb461b7d4d51e95d74c67/desktop/src/log-parser.js#L6-L58
[ss-tests]: https://github.com/noman-omen/poe2-splitstone/blob/f3c66aa6a5390413505fb461b7d4d51e95d74c67/desktop/tests/log-parser.test.js#L10-L12
[ss-noise]: https://github.com/noman-omen/poe2-splitstone/blob/f3c66aa6a5390413505fb461b7d4d51e95d74c67/desktop/tests/log-parser.test.js#L42-L48
[ss-lvl]: https://github.com/noman-omen/poe2-splitstone/blob/f3c66aa6a5390413505fb461b7d4d51e95d74c67/desktop/tests/log-parser.test.js#L51-L80
[ss-readme]: https://github.com/noman-omen/poe2-splitstone/blob/f3c66aa6a5390413505fb461b7d4d51e95d74c67/README.md#L63
[ss-changelog]: https://github.com/noman-omen/poe2-splitstone/blob/f3c66aa6a5390413505fb461b7d4d51e95d74c67/CHANGELOG.md#L7
[sg-parse]: https://github.com/ktu0216/poe2-saegida/blob/6796ddf3db2cb8040ae4a73bbbf5507bec4c09be/poe2_overlay/logparse.py#L11-L41
[sg-paths]: https://github.com/ktu0216/poe2-saegida/blob/6796ddf3db2cb8040ae4a73bbbf5507bec4c09be/poe2_overlay/config.py#L73-L101
[sg-ts]: https://github.com/ktu0216/poe2-saegida/blob/6796ddf3db2cb8040ae4a73bbbf5507bec4c09be/poe2_overlay/tracker.py#L84-L86
[sg-core]: https://github.com/ktu0216/poe2-saegida/blob/6796ddf3db2cb8040ae4a73bbbf5507bec4c09be/tests/test_core.py#L7-L35
[sg-afk]: https://github.com/ktu0216/poe2-saegida/blob/6796ddf3db2cb8040ae4a73bbbf5507bec4c09be/tests/test_timing.py#L13
[sg-dev]: https://github.com/ktu0216/poe2-saegida/blob/6796ddf3db2cb8040ae4a73bbbf5507bec4c09be/docs/DEVELOPMENT.md#L20-L40
[sg-i18n]: https://github.com/ktu0216/poe2-saegida/blob/6796ddf3db2cb8040ae4a73bbbf5507bec4c09be/poe2_overlay/i18n.py#L11-L52
[sg-manual]: https://github.com/ktu0216/poe2-saegida/blob/6796ddf3db2cb8040ae4a73bbbf5507bec4c09be/docs/MANUAL.en.md#L79
[mt-app]: https://github.com/igodspy/PoE2MT/blob/b4b4c2bff43c51f4a0ce21cb41af47f4bd2c0241/app.js#L405-L455
[mt-utf8]: https://github.com/igodspy/PoE2MT/blob/b4b4c2bff43c51f4a0ce21cb41af47f4bd2c0241/app.js#L303-L315
[mt-readme]: https://github.com/igodspy/PoE2MT/blob/b4b4c2bff43c51f4a0ce21cb41af47f4bd2c0241/README.md#L19-L44
[mt-size]: https://github.com/igodspy/PoE2MT/blob/b4b4c2bff43c51f4a0ce21cb41af47f4bd2c0241/index.html#L23
[mw-gen]: https://github.com/ugimser/mapwatch/blob/f3b5b9c3bae430d1311965755c026109b582b0c4/js/parsers.js#L13-L34
[mw-whisper]: https://github.com/ugimser/mapwatch/blob/f3b5b9c3bae430d1311965755c026109b582b0c4/js/parsers.js#L36-L72
[mw-open]: https://github.com/ugimser/mapwatch/blob/f3b5b9c3bae430d1311965755c026109b582b0c4/js/parsers.js#L74-L91
[mw-slain-level]: https://github.com/ugimser/mapwatch/blob/f3b5b9c3bae430d1311965755c026109b582b0c4/js/parsers.js#L110-L140
[mw-readme]: https://github.com/ugimser/mapwatch/blob/f3b5b9c3bae430d1311965755c026109b582b0c4/README.md#L9
[pn-watch]: https://github.com/buri34/poenavi/blob/5a485be8469a97c7f96f0bfd1b00794f6ec6436f/src/utils/log_watcher.py#L26-L38
[pn-readme]: https://github.com/buri34/poenavi/blob/5a485be8469a97c7f96f0bfd1b00794f6ec6436f/README.md#L78-L104
[pn-paths]: https://github.com/buri34/poenavi/blob/5a485be8469a97c7f96f0bfd1b00794f6ec6436f/src/utils/log_path_detector.py#L27-L44
[pn-fixtures]: https://github.com/buri34/poenavi/blob/5a485be8469a97c7f96f0bfd1b00794f6ec6436f/tests/test_hideout_notification.py#L64-L97
[qu-tags]: https://github.com/Vyary/qutm/blob/de642de860cee9284aba1ded9cdd0920b30555df/src/lib/File.ts#L39-L73
[qu-level]: https://github.com/Vyary/qutm/blob/de642de860cee9284aba1ded9cdd0920b30555df/src/lib/File.ts#L76-L81
[qu-path]: https://github.com/Vyary/qutm/blob/de642de860cee9284aba1ded9cdd0920b30555df/src/lib/File.ts#L131
[eu-loc]: https://github.com/Lailloken/Exile-UI/blob/67d1a3637e97b791072129f0cf890e22f385b237/Exile%20UI.ahk#L1192-L1210
[eu-tail]: https://github.com/Lailloken/Exile-UI/blob/67d1a3637e97b791072129f0cf890e22f385b237/Exile%20UI.ahk#L709-L744
[eu-parse]: https://github.com/Lailloken/Exile-UI/blob/67d1a3637e97b791072129f0cf890e22f385b237/modules/client%20log.ahk#L473-L495
[eu-meter]: https://github.com/Lailloken/Exile-UI/blob/67d1a3637e97b791072129f0cf890e22f385b237/modules/settings%20menu.ahk#L1061-L1065
[eu-removed]: https://github.com/Lailloken/Exile-UI/blob/67d1a3637e97b791072129f0cf890e22f385b237/modules/client%20log.ahk#L497-L498
[eu-backup]: https://github.com/Lailloken/Exile-UI/blob/67d1a3637e97b791072129f0cf890e22f385b237/modules/client%20log.ahk#L80-L170
[eu-lang]: https://github.com/Lailloken/Exile-UI/blob/67d1a3637e97b791072129f0cf890e22f385b237/data/english/client.txt#L36-L150
[eu-help]: https://github.com/Lailloken/Exile-UI/blob/67d1a3637e97b791072129f0cf890e22f385b237/data/english/help%20tooltips.json#L114-L120
[el-main]: https://github.com/HeartofPhos/exile-log-api/blob/a0e75c0c8ef4c55cc29b29f5796d365be88157ce/src/main.rs#L44-L52
[ee-watch]: https://github.com/Kvan7/Exiled-Exchange-2/blob/cca30662bf31eaf38bd711e2ec1a6b899a06c40e/main/src/host-files/GameLogWatcher.ts
[plm-chat]: https://github.com/klayveR/poe-log-monitor/blob/d61aa3334a0f6b1f1ca9cef8164e058d2b9b4349/resource/events.json#L28-L46
[plm-sys]: https://github.com/klayveR/poe-log-monitor/blob/d61aa3334a0f6b1f1ca9cef8164e058d2b9b4349/resource/events.json#L8-L91
[tx-map]: https://github.com/dermow/TraXile/blob/f86a83f15c72db9d89446e7f11781e44bef1d5c8/TraXile/TrX_EventMapping.cs#L84-L99
[ps-awk]: https://github.com/TwentyFiveEx/poeseed/blob/61366ed5fbf93727136779427268c7435c195147/poeseed#L32-L62
[ggg-dev]: https://www.pathofexile.com/developer/docs
[epic]: https://store.epicgames.com/p/path-of-exile-2
[sb]: https://www.switchbladegaming.com/path-of-exile-2/save-file-location-4/
[ahk]: https://www.autohotkey.com/docs/v1/lib/FileOpen.htm
