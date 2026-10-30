# ADR 0016: CLI output formats and exit codes

Status: accepted, 2026-10-23

**Context.** The CLI uses three exit codes (0 ok, 1 not found / no match, 2 usage or IO error), and `doctor` must exit cleanly on a machine without PoE2. Still open: what `doctor` returns when nothing is found, what `paths` prints without `--json`, the human-readable output, and how the CLI is tested.

**Decision.**
- **`doctor`** prints a table with the columns `KIND SOURCE EXISTS PATH NOTE`.
  - It has one row per candidate, in Discovery order. Columns are separated by two spaces, and trailing spaces are trimmed.
  - After the rows it prints `<n> of <m> locations exist.`, and a hint line when no client log exists.
  - With `--json` it prints the Discovery JSON, pretty-printed.
  - **It exits 0 even when nothing is found**, because a report that finds nothing is still a successful diagnosis.
- **`paths`** prints `<kind>\t<path>` for the first existing candidate of each kind (`installs`, `client_log`, `user_dir`, `build_planner`, `filters`). With `--json` it prints the Discovery JSON, pretty-printed. It exits 1 when no candidate exists, after printing.
- **`tail`** follows the file given with `--log PATH`, or else the first existing client log.
  - With neither, it exits 1. A path it can't open exits 2.
  - Once the file is open, it writes `following <path>` to stderr. It then polls every 250 ms until it is killed.
- **`parse FILE`** streams the whole file and exits 0. A file it can't read exits 2.
- **Event lines.** With `--json`, each event is one compact log-line JSON object on its own line (ADR 0012). Without it, the line is `<timestamp or -> <type> <key>=<JSON value> …`, with the event's other fields in key order, for example `2026-01-02T03:04:05 LevelUp class="Smith of Kitava" level=12 name="ExileOne"`.
- **Errors.** Usage errors exit 2, with clap's message on stderr. `--help` and `--version` exit 0. If stdout is closed (broken pipe), the command stops quietly with exit code 0.
- **Testing.** The binary is a thin `main` over the library function `poe2_cli::run(args, stdout, stderr) -> u8`, so tests run commands in-process, including a bounded-memory test of `parse`. That library is internal, not a stable API.

**Considered options.**
- `doctor` exits 1 when nothing is found: consistent with `paths`, but a clean exit should mean success, and `paths` is the scripting interface.
- `paths` prints only JSON: then its `--json` flag would do nothing.

**Consequences.** Scripts use `poe2 paths` (the exit code plus tab-separated lines) or `poe2 paths --json`. People use `poe2 doctor`.
