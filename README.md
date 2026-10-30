# poe2-core

Community tools for Path of Exile 2 each re-implement finding the game's files and reading its
`Client.txt`, and they disagree on the paths and the log format. poe2-core does it once, with tests,
for Windows and Linux: a Rust library, a `poe2` command-line tool, and Node and Python bindings.

## Demo: `poe2 doctor`

`poe2 doctor` lists every place it looked, whether each exists, and why it looked there. This run
used the synthetic native Steam install that [docs/demo.tape](docs/demo.tape) sets up in an empty
home folder (shown here as `/home/<user>`):

```console
$ poe2 doctor
KIND           SOURCE        EXISTS  PATH                                                                                                                                       NOTE
installs       Steam         yes     /home/<user>/.local/share/Steam/steamapps/common/Path of Exile 2                                                                           app manifest in library /home/<user>/.local/share/Steam
installs       Steam         no      /home/<user>/.steam/steam/steamapps/common/Path of Exile 2                                                                                 Steam root not found
installs       Steam         no      /home/<user>/.steam/root/steamapps/common/Path of Exile 2                                                                                  Steam root not found
installs       Steam         no      /home/<user>/.steam/debian-installation/steamapps/common/Path of Exile 2                                                                   Steam root not found
installs       Steam         no      /home/<user>/Steam/steamapps/common/Path of Exile 2                                                                                        Steam root not found
installs       FlatpakSteam  no      /home/<user>/.var/app/com.valvesoftware.Steam/.local/share/Steam/steamapps/common/Path of Exile 2                                          Steam root not found
installs       FlatpakSteam  no      /home/<user>/.var/app/com.valvesoftware.Steam/data/Steam/steamapps/common/Path of Exile 2                                                  Steam root not found
installs       Steam         no      /home/<user>/snap/steam/common/.local/share/Steam/steamapps/common/Path of Exile 2                                                         Steam root not found
installs       Steam         no      /home/<user>/.snap/data/steam/common/.local/share/Steam/steamapps/common/Path of Exile 2                                                   Steam root not found
installs       Heroic        no      /home/<user>/Games/Heroic/Prefixes/Path of Exile 2/drive_c/Program Files (x86)/Grinding Gear Games/Path of Exile 2                         Heroic default prefix
installs       Lutris        no      /home/<user>/Games/path-of-exile-2/drive_c/Program Files (x86)/Grinding Gear Games/Path of Exile 2                                         Lutris default prefix
installs       Fallback      no      /home/<user>/.wine/drive_c/Program Files (x86)/Grinding Gear Games/Path of Exile 2                                                         default Wine prefix
client_log     Steam         yes     /home/<user>/.local/share/Steam/steamapps/common/Path of Exile 2/logs/Client.txt                                                           app manifest in library /home/<user>/.local/share/Steam
client_log     Steam         no      /home/<user>/.steam/steam/steamapps/common/Path of Exile 2/logs/Client.txt                                                                 Steam root not found
client_log     Steam         no      /home/<user>/.steam/root/steamapps/common/Path of Exile 2/logs/Client.txt                                                                  Steam root not found
client_log     Steam         no      /home/<user>/.steam/debian-installation/steamapps/common/Path of Exile 2/logs/Client.txt                                                   Steam root not found
client_log     Steam         no      /home/<user>/Steam/steamapps/common/Path of Exile 2/logs/Client.txt                                                                        Steam root not found
client_log     FlatpakSteam  no      /home/<user>/.var/app/com.valvesoftware.Steam/.local/share/Steam/steamapps/common/Path of Exile 2/logs/Client.txt                          Steam root not found
client_log     FlatpakSteam  no      /home/<user>/.var/app/com.valvesoftware.Steam/data/Steam/steamapps/common/Path of Exile 2/logs/Client.txt                                  Steam root not found
client_log     Steam         no      /home/<user>/snap/steam/common/.local/share/Steam/steamapps/common/Path of Exile 2/logs/Client.txt                                         Steam root not found
client_log     Steam         no      /home/<user>/.snap/data/steam/common/.local/share/Steam/steamapps/common/Path of Exile 2/logs/Client.txt                                   Steam root not found
client_log     Heroic        no      /home/<user>/Games/Heroic/Prefixes/Path of Exile 2/drive_c/Program Files (x86)/Grinding Gear Games/Path of Exile 2/logs/Client.txt         Heroic default prefix
client_log     Lutris        no      /home/<user>/Games/path-of-exile-2/drive_c/Program Files (x86)/Grinding Gear Games/Path of Exile 2/logs/Client.txt                         Lutris default prefix
client_log     Fallback      no      /home/<user>/.wine/drive_c/Program Files (x86)/Grinding Gear Games/Path of Exile 2/logs/Client.txt                                         default Wine prefix
user_dir       ProtonPrefix  no      /home/<user>/.local/share/Steam/steamapps/compatdata/2694490/pfx/drive_c/users/steamuser/Documents/My Games/Path of Exile 2                compatdata/2694490 (store app); created on first launch
build_planner  ProtonPrefix  no      /home/<user>/.local/share/Steam/steamapps/compatdata/2694490/pfx/drive_c/users/steamuser/Documents/My Games/Path of Exile 2/BuildPlanner   compatdata/2694490 (store app); created on first launch
filters        ProtonPrefix  no      /home/<user>/.local/share/Steam/steamapps/compatdata/2694490/pfx/drive_c/users/steamuser/Documents/My Games/Path of Exile 2                compatdata/2694490 (store app); created on first launch
filters        ProtonPrefix  no      /home/<user>/.local/share/Steam/steamapps/compatdata/2694490/pfx/drive_c/users/steamuser/Documents/My Games/Path of Exile 2/OnlineFilters  compatdata/2694490 (store app); created on first launch
2 of 28 locations exist.
```

`doctor` exits 0 even when it finds nothing. For scripts, use `poe2 paths` (exit code 1 when no
candidate exists) or `poe2 paths --json`. The other commands are `poe2 parse FILE [--json]` and
`poe2 tail [--log PATH] [--from-start] [--json]`; `poe2 --help` lists them all.

## Install

poe2-core is not published to crates.io, npm or PyPI yet. Build it from a checkout of this
repository with Rust 1.99 or newer.

The `poe2` CLI:

```sh
cargo install --path crates/poe2-cli
```

The Rust library, as a path dependency in your `Cargo.toml`:

```toml
[dependencies]
poe2-core = { path = "path/to/poe2-core/crates/poe2-core" }
```

The Node binding (Node 20.17 or newer). Build it, then install the folder into your project:

```sh
cd bindings/node && npm ci && npm run build
cd your-project && npm install path/to/poe2-core/bindings/node
```

The Python binding (Python 3.9 or newer). Inside your virtual environment, this builds it with
maturin:

```sh
python -m pip install path/to/poe2-core/bindings/python
```

[CONTRIBUTING.md](CONTRIBUTING.md) has the full build and test steps.

## Library usage

Each example parses a line, finds the client log with `discover()`, then follows it. They use the
same values: a `Discovery` with five ranked lists (`installs`, `client_log`, `user_dir`,
`build_planner`, `filters`), and log lines with a `timestamp`, a `prefix` and an `event`. In Node and
Python these are plain objects and dicts in the JSON shape that `poe2 paths --json` and
`poe2 parse --json` print ([ADR 0012](docs/decisions/0012-serde-always-on-one-json-shape.md),
[ADR 0017](docs/decisions/0017-bindings-are-standalone-crates.md)). The full API is listed in
[ADR 0022](docs/decisions/0022-public-api-freeze.md).

### Rust

```rust
use poe2_core::locate::discover;
use poe2_core::log::{Event, LogTail, parse_line};
use std::{thread, time::Duration};

fn main() -> std::io::Result<()> {
    // Parse one line on its own.
    let line = "2026/01/02 03:04:05 123456789 1a2b3c4d [INFO Client 4242] : ExileOne (Smith of Kitava) is now level 12";
    if let Some(Event::LevelUp { name, level, .. }) = parse_line(line).map(|l| l.event) {
        println!("{name} reached level {level}");
    }

    // Each list is ranked, so the first client log that exists is the best guess.
    let found = discover();
    let Some(log) = found.client_log.iter().find(|c| c.exists) else {
        eprintln!("no client log found; set POE2_INSTALL_DIR to the absolute path of the game folder");
        return Ok(());
    };
    println!("following {}", log.value.display());

    // Follow the client log from its end. next_line() never blocks, so sleep when caught up.
    let mut tail = LogTail::open(&log.value)?;
    loop {
        match tail.next_line()? {
            Some(line) => match line.event {
                Event::ZoneEntered { area_name, .. } => println!("entered {area_name}"),
                Event::WhisperReceived { sender, text, .. } => println!("@{sender}: {text}"),
                _ => {}
            },
            None => thread::sleep(Duration::from_millis(250)),
        }
    }
}
```

`Event`, `Source` and `LogLine` are `#[non_exhaustive]`, so keep a `_` arm and `..` in patterns.

### Node

```js
const { discover, parseLine, LogTail } = require('poe2-core');

// Parse one line on its own.
const line = parseLine('2026/01/02 03:04:05 123456789 1a2b3c4d [INFO Client 4242] : ExileOne (Smith of Kitava) is now level 12');
if (line?.event.type === 'LevelUp') console.log(`${line.event.name} reached level ${line.event.level}`);

// Each list is ranked, so the first client log that exists is the best guess.
const log = discover().client_log.find((c) => c.exists);
if (!log) {
  console.error('no client log found; set POE2_INSTALL_DIR to the absolute path of the game folder');
  process.exit(1);
}
console.log(`following ${log.value}`);

// Follow the client log from its end. Iterating yields the lines written so far and stops when
// caught up, so iterate again on a timer.
const tail = new LogTail(log.value);
setInterval(() => {
  for (const { event } of tail) {
    if (event.type === 'ZoneEntered') console.log(`entered ${event.area_name}`);
    else if (event.type === 'WhisperReceived') console.log(`@${event.sender}: ${event.text}`);
  }
}, 250);
```

### Python

```python
import poe2_core

# Parse one line on its own.
line = poe2_core.parse_line("2026/01/02 03:04:05 123456789 1a2b3c4d [INFO Client 4242] : ExileOne (Smith of Kitava) is now level 12")
if line and line["event"]["type"] == "LevelUp":
    print(f"{line['event']['name']} reached level {line['event']['level']}")

# Each list is ranked, so the first client log that exists is the best guess.
log = next((c for c in poe2_core.discover()["client_log"] if c["exists"]), None)
if log is None:
    raise SystemExit("no client log found; set POE2_INSTALL_DIR to the absolute path of the game folder")
print(f"following {log['value']}")

# Follow the client log from its end. Iterating blocks until the next line arrives
# (polling every 250 ms); next_line() is the non-blocking form.
for line in poe2_core.LogTail(log["value"]):
    event = line["event"]
    if event["type"] == "ZoneEntered":
        print(f"entered {event['area_name']}")
    elif event["type"] == "WhisperReceived":
        print(f"@{event['sender']}: {event['text']}")
```

## Where it looks

poe2-core reads only these locations. A path in a launcher's records is followed only if it is
absolute; on Windows that means a drive path (`X:\…`), and UNC and other `\\…` paths are skipped
([ADR 0024](docs/decisions/0024-no-unc-paths-from-launcher-data.md)). Every environment variable below
is used only if it holds an absolute path; a relative value counts as unset, so the Windows ones fall
back to `C:\Program Files (x86)`, `C:\Program Files` and `C:\ProgramData`, `%USERPROFILE%` and
`$WINEPREFIX` are skipped, and each `${XDG_…:-…}` uses its default
([ADR 0025](docs/decisions/0025-relative-env-paths-are-ignored.md)).

Inside each Wine or Proton prefix (and its `pfx/` folder) it probes
`drive_c/Program Files (x86)/Grinding Gear Games/Path of Exile 2` and
`drive_c/Program Files/Grinding Gear Games/Path of Exile 2` for the install, and
`drive_c/users/*/Documents/My Games/Path of Exile 2` (or `My Documents`) for the user dir. The client log is
`<install>/logs/Client.txt`; the build planner folder is `<user dir>/BuildPlanner`, and the filter
folders are the user dir and its `OnlineFilters` folder.

- **Both OSes:** `POE2_INSTALL_DIR` and `POE2_USER_DIR`, when set to an absolute path (a UNC path is
  allowed); they always come first.
- **Windows:**
  - Steam: the registry values `HKCU\Software\Valve\Steam\SteamPath`,
    `HKLM\SOFTWARE\WOW6432Node\Valve\Steam\InstallPath` and
    `HKLM\SOFTWARE\Valve\Steam\InstallPath`, then `%ProgramFiles(x86)%\Steam`; in each root, both `libraryfolders.vdf` files and each library's
    `appmanifest_2694490.acf`, for at most 64 libraries per root
    ([ADR 0026](docs/decisions/0026-steam-library-list-cap.md)).
  - Standalone: `HKCU\Software\GrindingGearGames\Path of Exile 2\InstallLocation`, then
    `Grinding Gear Games\Path of Exile 2` under `%ProgramFiles(x86)%` and `%ProgramFiles%`.
  - Epic: the `*.item` manifests in `%PROGRAMDATA%\Epic\EpicGamesLauncher\Data\Manifests`.
  - User dir: `My Games\Path of Exile 2` under the Documents known folder (which follows OneDrive),
    then under `%USERPROFILE%\Documents`.
- **Linux:**
  - Steam roots: `~/.steam/steam`, `~/.steam/root`, `${XDG_DATA_HOME:-~/.local/share}/Steam`,
    `~/.steam/debian-installation`, `~/Steam`, Snap's `~/snap/steam/common/.local/share/Steam` and
    `~/.snap/data/steam/common/.local/share/Steam`, and Flatpak's
    `~/.var/app/com.valvesoftware.Steam/.local/share/Steam` and `…/data/Steam`. In each root, both
    `libraryfolders.vdf` files and each library's `appmanifest_2694490.acf`, for at most 64 libraries
    per root ([ADR 0026](docs/decisions/0026-steam-library-list-cap.md)).
  - Proton prefixes: every `steamapps/compatdata/*` in every Steam library, not only `2694490`, so
    non-Steam shortcuts are found too. Steam's `userdata/` folder is never read.
  - Heroic: `GamesConfig/*.json` and `config.json` in `${XDG_CONFIG_HOME:-~/.config}/heroic` and
    `~/.var/app/com.heroicgameslauncher.hgl/config/heroic`, then every prefix in
    `~/Games/Heroic/Prefixes` and `~/Games/Heroic/Prefixes/default`.
  - Lutris: `lutris/games/*.yml` in `${XDG_CONFIG_HOME:-~/.config}` and
    `${XDG_DATA_HOME:-~/.local/share}`, then `~/Games/path-of-exile-2`.
  - Bottles: every bottle in `${XDG_DATA_HOME:-~/.local/share}/bottles/bottles` and
    `~/.var/app/com.usebottles.bottles/data/bottles/bottles`, following each `placeholder.yml` and the
    `custom_bottles_path` in `data.yml`.
  - Wine: `$WINEPREFIX`, `~/.wine` and every `~/Games/umu/*`.

Flatpak paths are tried both as recorded and remapped into the app's folder
([ADR 0004](docs/decisions/0004-flatpak-path-remapping.md)). The sources for each location are in
[docs/research/windows-paths.md](docs/research/windows-paths.md) and
[docs/research/linux-paths.md](docs/research/linux-paths.md).

## Platform matrix

"Tested in CI" means the search logic runs against synthetic layouts in an in-memory fake system,
with Windows and Linux layouts tested on both `ubuntu-latest` and `windows-latest`
([ci.yml](.github/workflows/ci.yml)). On Linux CI, the CLI tests also run the real `discover()`
against a synthetic native Steam install in an empty temporary home. The Node binding (Node 20.17
and 22, [node.yml](.github/workflows/node.yml)) and the Python binding
([python.yml](.github/workflows/python.yml)) are built and smoke-tested on both runners too.
poe2-core has been checked only on a real Linux install with native Steam and Proton, where
`poe2 doctor` and `poe2 paths` found the install, Client.txt, the user
dir, the build planner folder and the filter folders. "Untested" lists the layouts that come from
launcher source code or a single source and that no test fixture has been checked against.

| Source | OS | Tested in CI | Tested on a real install | Untested |
|---|---|---|---|---|
| `EnvOverride` | both | yes | not yet | |
| `Steam` | Windows | yes: registry roots, both library lists, app manifests | not yet | |
| `Steam` | Linux | yes: native, Debian and Snap roots, `XDG_DATA_HOME`, both library lists; plus the real-filesystem CLI test | yes (a real Linux install with native Steam and Proton) | whether Snap's library list holds host paths; `SteamApps` capitalisation in old libraries (single source) |
| `ProtonPrefix` | Linux | yes: store and shortcut prefixes in every library | yes (a real Linux install with native Steam and Proton) | |
| `FlatpakSteam` | Linux | yes, including the path remaps | not yet | the forms of the library `"0"` path, and libraries added under `$HOME` through overrides (remap inferred, [ADR 0004](docs/decisions/0004-flatpak-path-remapping.md)) |
| `Standalone` | Windows | yes | not yet | the `InstallLocation` registry value rests on one source |
| `Epic` | Windows | yes | not yet | PoE2's Epic `AppName`s rest on one source; no real `.item` manifest checked |
| `Heroic` | Linux | yes, native and Flatpak | not yet | Flatpak prefixes outside `~/Games/Heroic` (remap inferred) |
| `Lutris` | Linux | yes | not yet | |
| `Bottles` | Linux | yes, native and Flatpak | not yet | Flatpak paths recorded as `/var/data/…` (remap inferred); whether Bottles' YAML folds like Lutris's |
| `KnownFolder` | Windows | yes | not yet | |
| `Fallback` | both | yes: default Steam and standalone folders, `%USERPROFILE%\Documents`, Wine, umu | not yet | whether the standalone installer's default folder is still current |

The ticket [linux-launcher-layouts](docs/tickets/linux-launcher-layouts.md) asks for real copies
of the untested Linux layouts. If your install isn't found, please open an issue with the output of
`poe2 paths --json`, with your username and paths anonymised.

## Plays by GGG's rules

poe2-core is a passive file reader. It reads launcher records and the client log, and nothing else:
no overlay or GUI, no reading of game memory, no input automation, no screen reading, no network
calls, and no pricing or trade features. It never writes files or starts the game. GGG's developer
documentation says:

> "Reading the game's log files is okay as long as the user is aware of what you are doing with
> that data."
>
> ([GGG developer docs](https://www.pathofexile.com/developer/docs))

If you build on poe2-core, tell your users what you do with their log.

## English only in v1

v1 recognises system messages from the English client only
([ADR 0009](docs/decisions/0009-english-only-v1-locale-tables.md)). In other languages, zone
entered, level-up, slain and AFK lines parse as `Unknown`, with the line kept verbatim, and so do
whispers if the client translates the `@From` and `@To` markers (nobody knows yet). The
engine lines (`LogOpened`, `AreaGenerated`, `SceneSet`) are English in every client and parse
everywhere. The ticket [locale-tables](docs/tickets/locale-tables.md) tracks other languages.

## Not affiliated with Grinding Gear Games

poe2-core is a community project. It is not affiliated with, endorsed by or sponsored by Grinding
Gear Games. Path of Exile is a trademark of Grinding Gear Games.

## Licence

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT)
at your option. Unless you explicitly state otherwise, any contribution intentionally submitted for
inclusion in this project by you, as defined in the Apache-2.0 license, shall be dual licensed as
above, without any additional terms or conditions.

See [SECURITY.md](SECURITY.md) to report a vulnerability, [CONTRIBUTING.md](CONTRIBUTING.md) to
contribute, and [CHANGELOG.md](CHANGELOG.md) for changes.
