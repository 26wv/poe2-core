# ADR 0013: Corpus snapshots are the exact output of `poe2 parse --json`

Status: accepted, 2026-10-23

**Context.** Each `tests/corpus/*.txt` needs an expected JSON snapshot next to it, and the CLI tests check against the same files. insta is the usual tool, but it keeps `.snap` files with a YAML header in a `snapshots/` folder, so its files are not what the CLI prints.

**Decision.**
- `tests/corpus/<name>.txt` is a synthetic log. `tests/corpus/<name>.json` next to it holds exactly the bytes that `poe2 parse tests/corpus/<name>.txt --json` writes: one compact JSON object per non-blank line (ADR 0012), each followed by `\n`. The file is JSON Lines despite its `.json` extension.
- `crates/poe2-core/tests/corpus.rs` checks every pair with `read_lines` and `serde_json::to_string`, and it reports the first line that differs. With `UPDATE_SNAPSHOTS=1` set, it rewrites the `.json` files instead, and the diff is read before committing.
- The same test enforces the corpus rules:
  - each `<event>.txt` (`level_up.txt` and so on) holds only lines of that event;
  - `unknown.txt` holds only unknown lines, and each one's `raw` equals its source line;
  - `session.txt` is a mixed excerpt;
  - every v1 event has its file.
- The CLI's tests run `poe2 parse --json` on every `.txt` file and compare the output with the `.json` bytes.
- `.gitattributes` marks `tests/corpus/**` as `-text`, so CRLF fixtures and snapshots come through Windows checkouts unchanged.

**Considered options.** insta brings a dependency tree, and the CLI's output can't be compared byte for byte with its snapshot files.

**Consequences.** About 40 lines of test code replace a dependency. A change to the JSON shape shows up as a diff in every snapshot.
