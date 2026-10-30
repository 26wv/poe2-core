# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-10-24

The first version. It is not yet published to crates.io, npm or PyPI. Its public API is frozen by
[ADR 0022](docs/decisions/0022-public-api-freeze.md).

### Added

- `poe2_core::locate::discover()`, which returns a ranked `Discovery` of candidates for the install,
  client log, user dir, build planner folder and filter folder. It only reads files and never
  launches or changes anything. Each candidate names its `Source`:
  - `EnvOverride`: `POE2_INSTALL_DIR` or `POE2_USER_DIR`;
  - `Steam`: a Steam root and its libraries;
  - `ProtonPrefix`: a Proton prefix in a Steam library;
  - `FlatpakSteam`: the Flatpak Steam root and its libraries;
  - `Standalone`: a standalone install;
  - `Epic`: the Epic Games Launcher's manifests;
  - `Heroic`: Heroic's records;
  - `Lutris`: Lutris's records;
  - `Bottles`: Bottles' records;
  - `KnownFolder`: the Windows Documents known folder;
  - `Fallback`: a documented default location.
- `poe2_core::log::parse_line`, which turns a client log line into a `LogLine` with its timestamp,
  prefix and one of the v1 events ([ADR 0007](docs/decisions/0007-v1-event-set.md)):
  - `LogOpened`: the `***** LOG FILE OPENING *****` line;
  - `AreaGenerated`: area level, area id and seed;
  - `SceneSet`: the area now shown;
  - `ZoneEntered`: `You have entered <area>.`;
  - `LevelUp`: name, class and level;
  - `CharacterSlain`: a character died;
  - `WhisperReceived`: `@From <name>: <text>`;
  - `WhisperSent`: `@To <name>: <text>`;
  - `AfkOn`, with an optional autoreply, and `AfkOff`;
  - `Unknown`: any other non-blank line, kept verbatim ([ADR 0008](docs/decisions/0008-unknown-lines-are-preserved.md)).
- `read_lines`, which streams a whole log in 64 KiB chunks, and `LogTail`, which follows a client log
  through a non-blocking `next_line` that copes with truncation and partial lines
  ([ADR 0014](docs/decisions/0014-logtail-non-blocking-pull.md)).
- One JSON shape for a `Discovery` and a `LogLine`, shared by the CLI and the bindings
  ([ADR 0012](docs/decisions/0012-serde-always-on-one-json-shape.md)).
- The `poe2` CLI ([ADR 0016](docs/decisions/0016-cli-output-and-exit-codes.md)):
  - `poe2 doctor [--json]`: a table of every candidate it checked, and whether each one exists;
  - `poe2 paths [--json]`: the first existing candidate of each kind;
  - `poe2 parse FILE [--json]`: the events in a log file;
  - `poe2 tail [--log PATH] [--from-start] [--json]`: follows the client log.
- Node binding (`bindings/node`, napi-rs) with `discover()`, `parseLine()` and `LogTail`, returning
  plain objects equal to `JSON.parse` of the CLI's JSON. Every number is a JS `number`
  ([ADR 0023](docs/decisions/0023-node-numbers-follow-json-parse.md)).
- Python binding (`bindings/python`, PyO3 and maturin) with `discover()`, `parse_line()` and an
  iterable `LogTail`, returning dicts in the same shape with exact ints.

### Changed

- None.

### Security

- Windows paths from launcher data (the registry, `libraryfolders.vdf`, Epic manifests) are used only
  as drive paths (`X:\…`); UNC and other `\\…` paths are skipped. Values the user or the OS
  sets may still be UNC: the `POE2_*` overrides, the other environment path values and the Documents
  folder ([ADR 0024](docs/decisions/0024-no-unc-paths-from-launcher-data.md)).
- A relative value of an environment path variable counts as unset (`POE2_INSTALL_DIR`,
  `POE2_USER_DIR`, `WINEPREFIX`, `ProgramFiles`, `ProgramFiles(x86)`, `PROGRAMDATA`, `USERPROFILE`,
  `XDG_CONFIG_HOME`, `XDG_DATA_HOME`) ([ADR 0025](docs/decisions/0025-relative-env-paths-are-ignored.md)).
- A Steam root takes at most 64 libraries from its library lists, noted `library list truncated to 64
  entries` ([ADR 0026](docs/decisions/0026-steam-library-list-cap.md)).
- Launcher-file reads and `LogTail` open only regular files: a FIFO, device or folder fails with
  `InvalidInput` "not a regular file". `poe2 parse FILE` still opens whatever path it is given (and
  `read_lines` reads any reader), so a FIFO named there blocks by design.

### Known gaps

- AFK autoreply text in English PoE2 lines is unconfirmed ([ticket](docs/tickets/afk-autoreply-text.md)).
- Binding code polish: error kinds, json.loads, unsafe lint ([ticket](docs/tickets/binding-code-polish.md)).
- Binding and crate packaging gaps before publishing ([ticket](docs/tickets/binding-packaging.md)).
- Binding test gaps ([ticket](docs/tickets/binding-test-gaps.md)).
- candidates child(): exact join first, folders only ([ticket](docs/tickets/candidates-child-lookup.md)).
- Lint the bindings in CI and widen the Python matrix ([ticket](docs/tickets/ci-coverage-gaps.md)).
- Pin CI actions to SHAs and build from the lockfiles ([ticket](docs/tickets/ci-supply-chain-pinning.md)).
- Escape control characters and pad by display width in human CLI output ([ticket](docs/tickets/cli-human-output-escaping.md)).
- CLI test gaps and read-error flush ([ticket](docs/tickets/cli-test-gaps.md)).
- Scan dependencies for advisories ([ticket](docs/tickets/dependency-advisory-scanning.md)).
- DND on and off lines are parsed as `Unknown` ([ticket](docs/tickets/dnd-toggle.md)).
- Fix stale ADR citations and wording ([ticket](docs/tickets/docs-citations-wording.md)).
- PoE2's Epic identifiers and paths have one source each ([ticket](docs/tickets/epic-identifiers.md)).
- Treat a relative HOME as unset ([ticket](docs/tickets/home-relative-filter.md)).
- Launcher file parsers: fuzz alphabets, YAML null, memory notes ([ticket](docs/tickets/launcher-parser-hardening.md)).
- Several Linux launcher layouts are inferred, not observed ([ticket](docs/tickets/linux-launcher-layouts.md)).
- Only English locale tables ship ([ticket](docs/tickets/locale-tables.md)).
- Locator test gaps ([ticket](docs/tickets/locate-test-gaps.md)).
- Prefer regular files when choosing the client log ([ticket](docs/tickets/log-choice-regular-files.md)).
- Log parser: one digits helper and test gaps ([ticket](docs/tickets/log-parser-test-gaps.md)).
- Log reader and tail: shared chunk loop, bounds and test gaps ([ticket](docs/tickets/log-reader-robustness.md)).
- Log files other than `Client.txt` are not listed ([ticket](docs/tickets/other-log-files.md)).
- The meaning of some prefix fields is unconfirmed ([ticket](docs/tickets/prefix-field-meanings.md)).
- Guard every package against accidental publishing ([ticket](docs/tickets/publish-guards.md)).
- Python discover() reads env with the GIL released ([ticket](docs/tickets/python-discover-env-race.md)).
- Share the read-and-note and XDG helpers across the launcher searches ([ticket](docs/tickets/read-and-note-helper.md)).
- One rule for where launcher read errors are noted ([ticket](docs/tickets/read-error-notes-rule.md)).
- Close the regular-file check-then-open race ([ticket](docs/tickets/regular-file-open-race.md)).
- SECURITY.md scope and the mtime ranking ([ticket](docs/tickets/security-md-scope.md)).
- The standalone install's registry key has a single source ([ticket](docs/tickets/standalone-registry-key.md)).
- Reject Windows look-alike installdir names ([ticket](docs/tickets/steam-installdir-trailing-dots.md)).
- The suicide message is parsed as `Unknown` ([ticket](docs/tickets/suicide-message.md)).
- Test support: FakeSystem edges, temp dirs, known-limit comments ([ticket](docs/tickets/test-support-hygiene.md)).
- A guild tag in a whisper would end up in the name ([ticket](docs/tickets/whisper-guild-tag.md)).
- The whisper-sent line has little PoE2 evidence ([ticket](docs/tickets/whisper-sent-evidence.md)).
- Test the Win32 reads on the Windows runner ([ticket](docs/tickets/win32-registry-tests.md)).
- Reject Windows DOS device names in launcher paths ([ticket](docs/tickets/windows-dos-device-names.md)).
