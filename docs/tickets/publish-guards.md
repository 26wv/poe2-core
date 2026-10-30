# Guard every package against accidental publishing

**Context.** Found in the v0.1.0 code review. Nothing is published yet. npm has `"private": true` and both binding crates have `publish = false`, but crates/poe2-core/Cargo.toml and crates/poe2-cli/Cargo.toml have no `publish = false`, and bindings/python/pyproject.toml has no guard.

**Problem.** One `cargo publish` or `maturin publish` would ship, and a crates.io release can't be undone (it would also ship the YAML fixtures and `gen_yaml.py`).

**Acceptance criteria.**
- Both workspace crates set `publish = false`.
- pyproject.toml has `classifiers = ["Private :: Do Not Upload"]`.
- The `binding-packaging` ticket notes that these guards are removed on purpose, as the last step before publishing.

**Rank.** should
