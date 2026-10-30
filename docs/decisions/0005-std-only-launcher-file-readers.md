# ADR 0005: Read launcher files with small std-only readers

Status: accepted, 2026-10-23

Amended by [ADR 0018](0018-malformed-launcher-files-are-skipped.md): malformed launcher files are skipped without a note.

**Context.** The locator reads three formats:
- KeyValues: library lists and app manifests;
- JSON: Epic manifests, and Heroic's `GamesConfig/*.json` and `config.json`;
- YAML: Lutris `games/*.yml`, and Bottles `placeholder.yml` and `data.yml`.

Lutris writes YAML with PyYAML `safe_dump`. That folds long plain scalars onto continuation lines (the default PoE2 exe path breaks after `Grinding Gear`), quotes and escapes non-ASCII text, and repeats a nested `game:` under `script:`. A naive line scan is therefore wrong ([linux-paths.md](../research/linux-paths.md) Q9). The YAML crates are in poor shape: `serde_yaml` is archived, `serde_yml` has RUSTSEC-2025-0068, and `yaml-rust` is unmaintained. `yaml-rust2` and `saphyr` are maintained, but they are full YAML parsers, brought in for three keys. The project prefers std to new dependencies, and a hand-written parser is fine if it is small and tested.

**Decision.** No parsing dependencies. The locator has three small readers, each with fixtures:
- **KeyValues**: a tokenizer for quoted strings and nested braces.
  - Key lookup ignores case.
  - It unescapes `\\`, `\"`, `\n` and `\t`.
  - It accepts both library list layouts: objects with `path`, and legacy plain strings. It takes only digit keys and skips a library whose `mounted` is present and not `"1"` ([windows-paths.md](../research/windows-paths.md) Q2).
- **JSON**: a token scanner with no tree. It reads strings, punctuation and literals, finds an object key by exact name, and returns its string or boolean value.
  - It unescapes `\" \\ \/ \b \f \n \r \t \uXXXX` and joins surrogate pairs.
  - It is used only for keys that occur once per file: `winePrefix`, `InstallLocation`, `CatalogNamespace`, `DisplayName` and `bIsIncompleteInstall`. Unlike a line scanner, it doesn't depend on how the file is indented.
- **YAML**: a scoped scalar reader. It returns one value under a top-level key: `game` → `prefix` for Lutris, `Path` in Bottles' `placeholder.yml`, and `custom_bottles_path` in Bottles' `data.yml`.
  - It handles plain, single-quoted and double-quoted scalars and their continuation lines.
  - It ignores everything outside the requested top-level key, so `script: … game:` never matches.
  - Its fixtures are generated once with PyYAML and committed.

  `game.exe` is not read. The install is probed inside the prefix instead (ADR 0003).

All three readers are panic-free on any input and work on size-capped reads (ADR 0002). On malformed input they return "not found" with a note, never an error that stops discovery.

**Considered options.** `serde_json` is maintained; reconsider it if JSON needs grow. For YAML, reconsider `yaml-rust2` or `saphyr` if needs grow, never `serde_yaml`, `serde_yml` or `yaml-rust`. `steamlocate` brings its own VDF stack and reads the real machine, bypassing `System`.

**Consequences.** We own a few hundred lines of parsing, which get the same arbitrary-input no-panic test as the log parser. If a launcher changes its writer, the fixtures, not a crate, show the break.
