# ADR 0012: Serde is always on, and one JSON shape serves the CLI, the snapshots and the bindings

Status: accepted, 2026-10-23

**Context.** Three consumers need the locator's and the parser's output as data: `poe2 … --json`, the corpus snapshots and the Node and Python bindings. Each could get its own shape, and a cargo feature could keep serde out of builds that don't want it.

**Decision.**
- `poe2-core` depends on `serde` with `derive`, and no feature flag turns it off. `LogLine`, `Prefix`, `Timestamp`, `Event`, `Discovery`, `Found` and `Source` implement `Serialize`. `serde_json` is only a dev-dependency of the core.
- Those derives define the JSON shape, and every consumer uses the same one:
  - A log line is `{"timestamp": …, "prefix": …, "event": …}`.
    - `timestamp` is `"YYYY-MM-DDTHH:MM:SS"` or `null`. It is local time with no zone (ticket `prefix-field-meanings`).
    - `prefix` is `{"ms_counter", "hex_id", "log_level", "log_source", "pid"}` or `null`.
    - `event` is internally tagged: `{"type": "<variant>", …fields}`.
  - The event fields are:
    - `AreaGenerated`: `area_level`, `area_id`, `seed`;
    - `SceneSet` and `ZoneEntered`: `area_name`;
    - `LevelUp`: `name`, `class`, `level`;
    - `CharacterSlain`: `name`;
    - `WhisperReceived`: `sender`, `text`;
    - `WhisperSent`: `recipient`, `text`;
    - `AfkOn`: `autoreply` (a string or `null`);
    - `Unknown`: `raw`;
    - `LogOpened` and `AfkOff`: none.
  - A Discovery is `{"installs", "client_log", "user_dir", "build_planner", "filters"}`. Each list holds candidates of the form `{"value": "<path>", "source": "<Source variant>", "exists": bool, "note": string or null}`.
  - A missing value is `null`, never left out.
- Every Discovery path is valid UTF-8, so serialising a Discovery never fails. This holds because the System returns paths as strings and skips directory entries that aren't UTF-8.

**Considered options.**
- A `serde` feature: it saves little, since serde is in nearly every dependency tree, and every test that touches JSON would need both feature settings.
- Hand-written JSON: more code for the same shape.
- A separate shape per consumer: three shapes to keep in step and document.

**Consequences.**
- Renaming a field or variant breaks scripts, snapshots and bindings at once. The public-API freeze ADR (ADR 0022) lists the shape.
- A new event variant is an additive change.
- JavaScript reads `seed` and `ms_counter` as doubles, so values above 2^53 lose precision there.
