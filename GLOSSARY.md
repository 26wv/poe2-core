# Glossary

Finds Path of Exile 2's files on Windows and Linux and parses its client log into typed events, by reading files only.
Use these words in code, docs and commits. A term followed by a `code` span is a code identifier; the others are vocabulary only.

## Game files

- **Install** (`Discovery::installs`): A folder holding one copy of the game: its executable and its `logs/` folder. Don't use: game dir, game folder, installation.

- **Client log** (`Discovery::client_log`): The game's append-only text log, `logs/Client.txt` inside an install. It is never in the user dir. Don't use: log file, chat log, game log.

- **User dir** (`Discovery::user_dir`): The game's per-user folder, `My Games/Path of Exile 2` inside a Documents folder, holding the game config, item filters and the build planner folder. Don't use: documents folder, config dir, save folder, My Games folder.

- **Documents folder**: The Windows Documents known folder, which may be redirected (for example to OneDrive), or a prefix user's `Documents` inside a Wine prefix. Don't use: My Documents (the legacy name).

- **Build planner folder** (`Discovery::build_planner`): The `BuildPlanner` folder in a user dir, holding the `*.build` files the game loads. Don't use: builds folder.

- **Filter folder** (`Discovery::filters`): A folder the game reads item filters (`*.filter`) from: the user dir itself, or its `OnlineFilters` child for filters synced from the website. Don't use: loot filter folder, filters dir.

## Launchers and Steam

- **Launcher**: A program that installs and starts the game: Steam, the Epic Games Launcher, Heroic, Lutris or Bottles. A standalone install starts itself. Don't use: store, platform, client.

- **Standalone install**: An install made by GGG's own installer, outside any launcher. Don't use: GGG launcher, non-Steam version.

- **Steam root**: The Steam client's own folder, holding `steamapps/` and `config/`. It is always a library as well. Don't use: Steam path, Steam dir, SteamPath (a registry value).

- **Library**: A folder whose `steamapps/` child holds Steam games. A Steam root's library list names the others. Don't use: library folder, Steam folder.

- **Library list**: `libraryfolders.vdf`, the KeyValues file in a Steam root that lists its libraries.

- **App manifest**: `steamapps/appmanifest_<app ID>.acf`, the KeyValues file for one installed app. Its `installdir` names the app's folder under `steamapps/common/`. Don't use: ACF; "manifest" on its own (Epic has manifests too).

- **KeyValues**: Valve's text format of quoted keys and values nested in braces, used by library lists and app manifests. Don't use: VDF (the name also covers a binary format).

- **App ID**: Steam's number for a store app. PoE2's is `2694490`. Don't use: game ID.

- **Shortcut ID**: The number Steam gives a non-Steam shortcut: bit 31 is set, and it differs per user and per shortcut. The `2315204395` in GGG's docs is one. Don't use: calling it PoE2's app ID.

- **Epic manifest**: An `.item` JSON file in the Epic Games Launcher's `Manifests` folder, one per installed Epic game.

## Wine prefixes

- **Wine prefix**: A folder that stands in for a Windows machine under Wine or Proton: a `drive_c/` folder plus registry files. Don't use: bottle (Bottles' word), WINEPREFIX (the variable).

- **Proton prefix**: The Wine prefix Proton creates on an app's first launch, at `steamapps/compatdata/<app ID or shortcut ID>/pfx` in a library. Don't use: compatdata (the parent folder, with one subfolder per app ID or shortcut ID run under Proton).

- **Prefix user**: The Windows user folder under a Wine prefix's `drive_c/users/`. It is `steamuser` under Proton and the Unix user name under plain Wine. Don't use: Wine user.

- **Sandbox path**: A path as a Flatpak app records it, seen from inside its sandbox. The same folder can have a different host path. Don't use: container path.

## Locating

- **Candidate** (`Found<T>`): One possible location, with its source, whether it exists, and an optional note on how it was found. Don't use: result, hit, match, guess.

- **Source** (`Source`): Where a candidate came from: an env override, a launcher's records (Steam, Flatpak Steam, a Proton prefix, standalone, Epic, Heroic, Lutris, Bottles), the Documents known folder, or a fallback. Don't use: origin, provider; "source" for the log source.

- **Env override** (`Source::EnvOverride`): A candidate taken from `POE2_INSTALL_DIR` or `POE2_USER_DIR`.

- **Fallback** (`Source::Fallback`): A candidate at a documented default location, not taken from any launcher's records. Don't use: default, guess.

- **Discovery** (`Discovery`): The ranked candidates from one locator run, with one list for each kind of location: installs, client logs, user dirs, build planner folders and filter folders. Don't use: scan, detection, result.

- **Ranking**: The order of each Discovery list: env overrides first, then candidates that exist, then (for installs and client logs) the most recently modified client log first. Don't use: priority, score.

- **Probe**: One check of a possible location through the System. Don't use: scan, detect.

- **System** (`System`): The trait that all platform probing goes through: the environment, the home folder, the Documents known folder, the registry, and files (exists, read, list, modified time, canonical path). Don't use: platform, OS layer, filesystem.

- **Target OS**: The operating system whose layout a System presents, Windows or Linux. Discovery searches that OS's locations. Don't use: host OS (a FakeSystem can present Windows on a Linux machine).

- **Real system**: The System that reads the actual machine, built for the host OS. Don't use: native system, live system.

- **FakeSystem** (`FakeSystem`): An in-memory System for tests. It presents a chosen target OS's files, links, registry and environment. Don't use: mock, stub.

## Client log lines

- **Log line** (`LogLine`): One non-blank line of a client log, parsed into its prefix fields (when it has them) and its event. Don't use: entry, record, message.

- **Prefix**: The fixed-format start of a line: `<date> <time> <ms counter> <hex id> [<log level> <log source> <pid>] `. Don't use: header, timestamp (only part of it).

- **Log level**: The first word inside a prefix's brackets (`INFO`, `DEBUG`). Don't use: "level" on its own (areas and characters have levels too).

- **Log source**: The second word inside a prefix's brackets. `Client` is the only value seen. Don't use: "source" on its own (that is a candidate's Source).

- **Body**: The text after the prefix. Don't use: message, payload.

- **System message**: A body that starts with `: `: a game notice such as a level-up or a death. It is localised, and players can't forge it. Don't use: server message, notification.

- **Chat line**: A body that starts with a channel marker (`#`, `$`, `%`, `&`, `@From `, `@To `) or with a speaker's name and `: `.

- **Engine line**: A body that is in fixed English in every client language, such as `Generating level …` or `[SCENE] Set Source […]`. Don't use: debug line.

- **Area**: One generated game location. Its area id is internal and never translated (`G1_2`), its area name is the localised display name, and its area level comes from `Generating level`. Don't use: zone, map, instance (except in the event name `ZoneEntered`).

- **Event** (`Event`): The typed meaning of a log line: one of the v1 events below, or an unknown line. Don't use: message, action.

- **Log opened** (`Event::LogOpened`): The `***** LOG FILE OPENING *****` line the game writes at each launch. It has a date and time but no other prefix fields.

- **Area generated** (`Event::AreaGenerated`): An engine line recording a new area instance, with its area level, area id and seed.

- **Scene set** (`Event::SceneSet`): An engine line naming the area now shown. The names `(null)` (loading) and `(unknown)` (login, character select) are not areas. Don't use: zone change.

- **Zone entered** (`Event::ZoneEntered`): The system message "You have entered <area name>." Not every load writes it.

- **Level-up** (`Event::LevelUp`): A system message that a character (yours or a party member's) has reached a new character level.

- **Character slain** (`Event::CharacterSlain`): A system message that a character (yours or a party member's) has died. Don't use: death (as a code name).

- **Whisper received**, **Whisper sent** (`Event::WhisperReceived`, `Event::WhisperSent`): A private chat line from (`@From`) or to (`@To`) another player.

- **AFK on**, **AFK off** (`Event::AfkOn`, `Event::AfkOff`): System messages that away-from-keyboard mode was switched on (with its autoreply, if one is logged) or off.

- **Unknown line** (`Event::Unknown { raw }`): A non-blank line that matches no v1 event. It keeps the whole line text as `raw`. Don't use: unparsed line, invalid line, garbage.

- **Locale table**: The list of message templates for one language (literal text with named placeholders) that recognises system messages and whisper markers. v1 has English only. Don't use: translation file, language pack.

## Following a log

- **LogTail** (`LogTail`): A follower of one client log path that yields the log lines appended to it. Don't use: watcher, monitor, reader.

- **Poll**: One check of the followed file for bytes beyond LogTail's read position.

- **Truncation**: The followed file becoming shorter than LogTail's read position, because it was cut or was replaced by a new file (rotation). Don't use: reset.

- **Partial line**: The text after the followed file's last line break: a line that is still being written. Don't use: fragment.
