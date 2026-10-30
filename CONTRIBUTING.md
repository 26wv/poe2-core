# Contributing to poe2-core

Thanks for helping. This guide covers the three things people most often contribute: running the tests,
adding a log line to the corpus and adding a launcher. Words such as *install*, *client log*, *user dir*,
*candidate* and *Discovery* are defined in [GLOSSARY.md](GLOSSARY.md), and design decisions are recorded
in [docs/decisions/](docs/decisions/).

## Running tests

The Rust workspace (the `poe2-core` library and the `poe2` CLI) needs Rust 1.99 or newer. Run these from
the repository root; every change has to pass all of them:

```sh
cargo fmt --all
cargo clippy --all-targets -- -D warnings
cargo test
```

If you touch `crates/poe2-core`, also check the Windows build, whatever OS you are on. This needs the
`x86_64-pc-windows-msvc` rustup target installed:

```sh
cargo check -p poe2-core --target x86_64-pc-windows-msvc
cargo clippy -p poe2-core --target x86_64-pc-windows-msvc -- -D warnings
```

The locator is tested against an in-memory fake system. On Linux the few CLI tests that run the real
`discover()` clear the whole environment and point `HOME` at an empty temporary folder, so they never
touch your real game files. On Windows those tests read the real registry, Steam library lists and
Documents folder (read-only) and check only the output's shape.

### Node binding

Needs Node 20.17 or newer. Everything installs into `bindings/node/node_modules`; never install
anything globally.

```sh
cd bindings/node
npm ci
npm run build
npm test
cargo fmt --manifest-path Cargo.toml
cargo clippy --manifest-path Cargo.toml -- -D warnings
```

### Python binding

Needs Python 3.9 or newer. Everything installs into a virtual environment in `bindings/python/.venv`;
never install anything globally. On Windows, run these in Git Bash, with `.venv/Scripts/` in place of
`.venv/bin/`.

```sh
cd bindings/python
python3 -m venv .venv
.venv/bin/python -m pip install maturin==1.15.0
VIRTUAL_ENV=$PWD/.venv .venv/bin/maturin develop
.venv/bin/python -m unittest discover -s tests -v
cargo fmt --manifest-path Cargo.toml
cargo clippy --manifest-path Cargo.toml -- -D warnings
```

Both bindings are separate Cargo workspaces, so the root `cargo test` doesn't build them. CI runs each
one in its own workflow (`.github/workflows/node.yml` and `.github/workflows/python.yml`).

## Adding a corpus line

`tests/corpus/` holds synthetic Client.txt excerpts. Next to each `<name>.txt` is a `<name>.json` file
holding exactly what `poe2 parse tests/corpus/<name>.txt --json` prints
([ADR 0013](docs/decisions/0013-corpus-snapshots-are-parse-json-output.md)). To add a line:

1. **Anonymise it.** Take **one line**; never paste or commit a whole Client.txt. Then:
   - no character, account or guild names: use `ExileOne`, `ExileTwo` (or `망명자` for a Korean
     client) and `<GLD>`;
   - no whisper text or whisper partners: use `TraderJoe` and short made-up text;
   - no IP addresses: use one from `192.0.2.0/24`;
   - no Steam IDs or other account numbers: remove them;
   - no usernames or home folders: use `<user>`, as in `/home/<user>` or `C:\Users\<user>`.

   Keep everything else (the timestamp, the prefix, the punctuation and spacing) exactly as it was. The
   parser depends on that shape, so a line with a "tidied" prefix tests nothing.
2. **Put it in the right file.** A line of a known event goes in `tests/corpus/<event>.txt`, for example
   `level_up.txt` or `zone_entered.txt`; each of these files holds only lines of its own event. A line
   that parses as `Unknown` goes in `tests/corpus/unknown.txt`. `tests/corpus/session.txt` is a mixed
   excerpt. Git keeps these files byte for byte (`.gitattributes` marks them `-text`), so don't let your
   editor change their line endings.
3. **Regenerate the snapshots:**

   ```sh
   UPDATE_SNAPSHOTS=1 cargo test -p poe2-core --test corpus
   ```

4. **Review the JSON diff** (`git diff tests/corpus/`). The new line should produce exactly the event and
   fields you expect, and no other line should change. Then run `cargo test -p poe2-core --test corpus`
   without the variable and commit the `.txt` and `.json` files together.

If the line parses wrongly, fix the parser in `crates/poe2-core/src/log/` in the same change; the
snapshot diff then shows what the fix changed.

## Adding a launcher

A launcher here is anything that records where Path of Exile 2 is installed (Steam, Epic, Heroic, Lutris,
Bottles and so on). Before you start, read [ADR 0002](docs/decisions/0002-system-trait-probing-seam.md)
(the `System` seam), [ADR 0005](docs/decisions/0005-std-only-launcher-file-readers.md) (launcher file
readers), [ADR 0015](docs/decisions/0015-fixed-locations-as-would-be-candidates.md) (would-be
candidates) and [ADR 0018](docs/decisions/0018-malformed-launcher-files-are-skipped.md) (malformed
files).

1. **Research first.** Write a note in `docs/research/`, next to `windows-paths.md` and `linux-paths.md`,
   saying where the launcher keeps its files and its Wine prefixes, with a source for every claim (the
   launcher's own docs or source code). Read only the launcher's documented folders. Never read Steam's
   `userdata/` folder, which is named after the Steam account ID
   ([ADR 0003](docs/decisions/0003-find-prefixes-by-enumeration.md)).
2. **Write an ADR if the design changes**, in `docs/decisions/`. A new `Source` variant is a change to
   the public API ([ADR 0022](docs/decisions/0022-public-api-freeze.md)) and needs one. Add no dependency
   without an ADR either: the existing readers in `crates/poe2-core/src/locate/` (`keyvalues.rs`,
   `json.rs` and `yaml.rs`) cover KeyValues, JSON and the YAML the launchers write.
3. **Add a search module** in `crates/poe2-core/src/locate/`, starting with a `//!` line naming the
   research sections and ADRs it implements. Every read of the environment, registry or filesystem goes
   through the `System` trait (`system.rs`); search code never calls `std::fs` or `std::env`, and builds
   paths as text with the helpers in `tpath.rs`. Wire the search into `discover_with` in
   `crates/poe2-core/src/locate.rs`, under the target OS it belongs to.
4. **Treat everything the launcher wrote as untrusted input:**
   - an empty value counts as absent;
   - a leading byte order mark is stripped before parsing;
   - a path that isn't absolute is skipped, never resolved against the working directory: filter it
     with `tpath::absolute` (on Windows only `X:\…`; `\\…` paths are skipped,
     [ADR 0024](docs/decisions/0024-no-unc-paths-from-launcher-data.md)), and read an env path with
     `System::env_path` ([ADR 0025](docs/decisions/0025-relative-env-paths-are-ignored.md));
   - reads stop at the 1 MiB read cap, and a malformed file is skipped and never stops discovery.
5. **Test it with `FakeSystem`** (`fake.rs`), in unit tests inside the module
   ([ADR 0011](docs/decisions/0011-locator-tests-are-unit-tests.md)): at least one test where the game is
   found, one where it isn't, and one with malformed launcher files. Use only the synthetic names and
   folders listed under [Privacy](#privacy).

## Privacy

Never commit a real Client.txt, real log lines, a username, a Steam ID, or character, account or guild
names, whispers or IP addresses. Fixtures are synthetic: the characters `ExileOne`, `ExileTwo` and
`망명자`, the trader `TraderJoe`, the guild tag `<GLD>`, IPs from `192.0.2.0/24`, and home folders written
`/home/<user>` and `C:\Users\<user>`. Contributed log lines are anonymised as described in
[Adding a corpus line](#adding-a-corpus-line). The same applies to issues and pull requests: never
attach output that shows your real username or paths.

## Commits

Use [Conventional Commits](https://www.conventionalcommits.org/): `feat:`, `fix:`, `docs:`, `test:`,
`chore:` and so on, followed by a short summary in the imperative, for example
`docs: explain the corpus file rules`. Keep each commit to one change, and make sure the
checks in [Running tests](#running-tests) pass before you open a pull request.
