# ADR 0022: The v1 public API is frozen

Status: accepted, 2026-10-24

**Context.** The Node and Python bindings, the CLI and outside code depend on the core API, so it is written down in one place. ADR 0011 keeps `System`, `FakeSystem` and `discover_with` crate-private, ADR 0012 fixes one JSON shape, ADR 0014 fixes `LogTail` and `read_lines`, ADR 0016 fixes the CLI, and ADR 0017 makes the bindings standalone crates that return the JSON shape. `cargo doc -p poe2-core --no-deps` documents exactly the items listed below and no others.

**Decision.** The v1 public API of `poe2-core` 0.1 is exactly what this ADR lists. Everything else in the crate is `pub(crate)` or private. Bindings, the CLI and outside code may rely only on what is listed here.

- **`poe2_core::locate`**

  ```rust
  pub fn discover() -> Discovery;

  #[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
  pub struct Discovery {
      pub installs: Vec<Found<PathBuf>>,
      pub client_log: Vec<Found<PathBuf>>,
      pub user_dir: Vec<Found<PathBuf>>,
      pub build_planner: Vec<Found<PathBuf>>,
      pub filters: Vec<Found<PathBuf>>,
  }

  #[derive(Debug, Clone, PartialEq, Eq, Serialize)]
  pub struct Found<T> {
      pub value: T,
      pub source: Source,
      pub exists: bool,
      pub note: Option<String>,
  }

  #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
  #[non_exhaustive]
  pub enum Source {
      EnvOverride, Steam, ProtonPrefix, FlatpakSteam, Standalone, Epic,
      Heroic, Lutris, Bottles, KnownFolder, Fallback,
  }
  ```

  Each list in a `Discovery` is ranked: env overrides first, then candidates that exist, then the most recently written client log.

- **`poe2_core::log`**

  ```rust
  pub fn parse_line(line: &str) -> Option<LogLine>;          // None only for a blank line
  pub fn read_lines<R: Read>(reader: R) -> ReadLines<R>;

  pub struct ReadLines<R> { /* private */ }
  impl<R: Read> Iterator for ReadLines<R> { type Item = io::Result<LogLine>; }

  pub struct LogTail { /* private */ }                       // also Debug, Send and Sync
  impl LogTail {
      pub fn open(path: impl Into<PathBuf>) -> io::Result<LogTail>;
      pub fn open_from_start(path: impl Into<PathBuf>) -> io::Result<LogTail>;
      pub fn next_line(&mut self) -> io::Result<Option<LogLine>>;
  }

  #[derive(Debug, Clone, PartialEq, Eq, Serialize)]
  #[non_exhaustive]
  pub struct LogLine {
      pub timestamp: Option<Timestamp>,
      pub prefix: Option<Prefix>,
      pub event: Event,
  }

  #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
  pub struct Timestamp {                                     // also Display and Serialize,
      pub year: u16, pub month: u8, pub day: u8,             // both as YYYY-MM-DDTHH:MM:SS
      pub hour: u8, pub minute: u8, pub second: u8,
  }

  #[derive(Debug, Clone, PartialEq, Eq, Serialize)]
  #[non_exhaustive]
  pub struct Prefix {
      pub ms_counter: u64,
      pub hex_id: String,
      pub log_level: String,
      pub log_source: String,
      pub pid: u32,
  }

  #[derive(Debug, Clone, PartialEq, Eq, Serialize)]
  #[serde(tag = "type")]
  #[non_exhaustive]
  pub enum Event {
      LogOpened,
      #[non_exhaustive] AreaGenerated { area_level: u32, area_id: String, seed: u64 },
      #[non_exhaustive] SceneSet { area_name: String },
      #[non_exhaustive] ZoneEntered { area_name: String },
      #[non_exhaustive] LevelUp { name: String, class: String, level: u32 },
      #[non_exhaustive] CharacterSlain { name: String },
      #[non_exhaustive] WhisperReceived { sender: String, text: String },
      #[non_exhaustive] WhisperSent { recipient: String, text: String },
      #[non_exhaustive] AfkOn { autoreply: Option<String> },
      AfkOff,
      #[non_exhaustive] Unknown { raw: String },
  }
  ```

  The behaviour of `LogTail` and `read_lines` is ADR 0014's: `next_line` never blocks, a missing file is no new data, lines are cut at 64 KiB.

  Every type listed here is `Send + Sync`; `Found<T>` and `ReadLines<R>` are when `T` or `R` is. The bindings hold `LogTail` across threads (a `#[pyclass]` must be `Sync`), and `crates/poe2-core/tests/send_sync.rs` fails to build if a type loses either.

- **JSON shape.** ADR 0012 defines it, byte-exact, from the serde derives above. A missing value is `null`, never left out. One line per shape and per event, taken from `tests/corpus/`:
  - Discovery: `{"installs":[…],"client_log":[…],"user_dir":[…],"build_planner":[…],"filters":[…]}`, each candidate like `{"value":"/home/<user>/.local/share/Steam/steamapps/common/Path of Exile 2","source":"Steam","exists":true,"note":null}`.
  - `LogOpened`: `{"timestamp":"2026-01-02T03:04:05","prefix":null,"event":{"type":"LogOpened"}}`
  - `AreaGenerated`: `{"timestamp":"2026-01-02T03:00:01","prefix":{"ms_counter":123457789,"hex_id":"1a2b3c4e","log_level":"DEBUG","log_source":"Client","pid":4242},"event":{"type":"AreaGenerated","area_level":1,"area_id":"G1_1","seed":2958305341}}`
  - `SceneSet`: `{"timestamp":"2026-01-02T03:00:06","prefix":{"ms_counter":123462789,"hex_id":"1a2b3c53","log_level":"INFO","log_source":"Client","pid":4242},"event":{"type":"SceneSet","area_name":"Clearfell"}}`
  - `ZoneEntered`: `{"timestamp":"2026-01-02T03:00:09","prefix":{"ms_counter":123465789,"hex_id":"1a2b3c56","log_level":"INFO","log_source":"Client","pid":4242},"event":{"type":"ZoneEntered","area_name":"Clearfell"}}`
  - `LevelUp`: `{"timestamp":"2026-01-02T03:00:13","prefix":{"ms_counter":123469789,"hex_id":"1a2b3c5a","log_level":"INFO","log_source":"Client","pid":4242},"event":{"type":"LevelUp","name":"ExileOne","class":"Smith of Kitava","level":12}}`
  - `CharacterSlain`: `{"timestamp":"2026-01-02T03:00:16","prefix":{"ms_counter":123472789,"hex_id":"1a2b3c5d","log_level":"INFO","log_source":"Client","pid":4242},"event":{"type":"CharacterSlain","name":"ExileOne"}}`
  - `WhisperReceived`: `{"timestamp":"2026-01-02T03:00:18","prefix":{"ms_counter":123474789,"hex_id":"1a2b3c5f","log_level":"INFO","log_source":"Client","pid":4242},"event":{"type":"WhisperReceived","sender":"TraderJoe","text":"Hi, is the bow still for sale?"}}`
  - `WhisperSent`: `{"timestamp":"2026-01-02T03:00:21","prefix":{"ms_counter":123477789,"hex_id":"1a2b3c62","log_level":"INFO","log_source":"Client","pid":4242},"event":{"type":"WhisperSent","recipient":"TraderJoe","text":"Yes, sending an invite."}}`
  - `AfkOn`: `{"timestamp":"2026-01-02T03:00:25","prefix":{"ms_counter":123481789,"hex_id":"1a2b3c66","log_level":"INFO","log_source":"Client","pid":4242},"event":{"type":"AfkOn","autoreply":"Back in 5"}}`
  - `AfkOff`: `{"timestamp":"2026-01-02T03:00:27","prefix":{"ms_counter":123483789,"hex_id":"1a2b3c68","log_level":"INFO","log_source":"Client","pid":4242},"event":{"type":"AfkOff"}}`
  - `Unknown`: `{"timestamp":null,"prefix":null,"event":{"type":"Unknown","raw":"hello, an unprefixed line"}}`
- **CLI** (ADR 0016). `poe2 doctor [--json]`, `poe2 paths [--json]`, `poe2 parse FILE [--json]`, `poe2 tail [--log PATH] [--from-start] [--json]`, plus `--help` and `--version`. Exit codes: 0 ok (`doctor` even when nothing is found; a closed stdout too), 1 not found (`paths` with no existing candidate, `tail` with no `--log` and no existing client log), 2 usage or IO error. The output formats are ADR 0016's. The `poe2_cli` library (`run`) is internal and not part of this API.
- **Additive changes**, allowed in a patch or minor release without a new ADR:
  - a new `Event` or `Source` variant (both are `#[non_exhaustive]`);
  - a new field on a `#[non_exhaustive]` type: `LogLine`, `Prefix`, or an `Event` struct variant; in the JSON it is a new key;
  - a new public function, type, method or trait impl.
- **Breaking changes**, which need an ADR that supersedes this one and a breaking version bump (0.1.x to 0.2.0 while the crate is below 1.0):
  - renaming or removing any item, field or variant listed here, or changing a signature;
  - any change to the fields of `Discovery`, `Found` or `Timestamp`, which are not `#[non_exhaustive]` and can be built with struct literals;
  - removing a derive or trait impl listed here, or a listed type losing `Send` or `Sync`;
  - any change to the JSON shape other than a new key or a new `type` or `source` value: a renamed or removed key, a changed value type, or a `null` left out;
  - a CLI command, flag or exit code renamed, removed or given a new meaning.

**Considered options.**
- Freeze only the bindings' surface (`discover`, `parse_line`, `LogTail`): the CLI and the snapshots already depend on the rest, so it would be frozen in practice anyway.
- Mark `Discovery` and `Found` `#[non_exhaustive]` now, so new kinds of location are additive: Rust callers could no longer build them with struct literals, and a change to their fields is meant to be breaking.

**Consequences.**
- Bindings build against this list and the JSON shape only; a binding that needs more asks for a new ADR first.
- A consumer must ignore unknown JSON keys and unknown `type` and `source` values, and Rust code must keep a `_` arm when matching `Event` or `Source`.
- A new kind of location in `Discovery` is a breaking change.
