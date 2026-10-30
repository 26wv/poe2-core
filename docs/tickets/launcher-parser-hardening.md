# Launcher file parsers: fuzz alphabets, YAML null, memory notes

**Context.** Found in the v0.1.0 code review.
- The `never_panics` proptests in locate/json.rs:223 and yaml.rs:298 use `any::<String>()`, so quote, `\u` and surrogate paths are barely fuzzed. json.rs:63 accepts raw control characters in strings.
- YAML `prefix: ~` comes back as `"~"` (yaml.rs:84 `plain`), which `expand_tilde` turns into `$HOME`, which is then probed as a Wine prefix (lutris.rs:44-46). `null` comes back as `"null"`.
- YAML tests miss the escapes `\\ \" \/ \n \t \U`, a BOM, content after a closing quote, and an indented comment after a quoted value; some malformed YAML is decoded rather than rejected.
- keyvalues.rs:74-96 builds a tree of about 30-40x the input (bounded by `READ_CAP`), undocumented. keyvalues.rs:6-7 doesn't say `MAX_DEPTH` is what stops the recursive drop of nested `Value`s overflowing the stack.

**Problem.** Trust-boundary parsers are fuzzed with the wrong alphabet, YAML null is read as a path, and two limits lack their reason.

**Acceptance criteria.**
- JSON and YAML proptests use a format-alphabet strategy, as keyvalues.rs's token soup does.
- `plain` maps `~`, `null`, `Null` and `NULL` to `None`, with a test.
- Tests cover the YAML cases above; the raw-control-character leniency is documented or rejected.
- keyvalues.rs documents the memory bound and the reason for `MAX_DEPTH`.

**Rank.** nice
