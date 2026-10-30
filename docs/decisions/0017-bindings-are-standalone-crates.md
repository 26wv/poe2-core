# ADR 0017: Bindings are standalone crates that return the core's JSON shape

Status: accepted, 2026-10-23

Amended by [ADR 0023](0023-node-numbers-follow-json-parse.md): Node builds its values with `JSON.parse`, not napi's `serde-json` feature.

**Context.** Version 1 includes Node (napi-rs) and Python (PyO3 + maturin) bindings with a tiny surface: `discover()`, `parse_line()` and a tail iterator. As members of the root workspace, the bindings would be built by `cargo test`, and their link requirements could break the core's test run. Mapping every Rust type to hand-written JS and Python classes would double the API that has to stay frozen.

**Decision.**
- `bindings/node/` and `bindings/python/` are each their own Cargo workspace, with an empty `[workspace]` table in their `Cargo.toml`. Each depends on `crates/poe2-core` by path, so the root `cargo test` never builds them. Each has its own lock file and its own CI workflow.
- Values are the core's JSON shape (ADR 0012) as native objects. Node gets them through napi's `serde-json` feature, which turns a `serde_json::Value` into a JS object. Python gets them through `json.loads` on the output of `serde_json::to_string`.
- Node surface:
  - `discover()`, and `parseLine(line)`, which returns an object or `null`.
  - `new LogTail(path, fromStart?)`, whose `nextLine()` returns an object or `null` and throws on IO errors.
  - `for (const line of tail)` yields the lines available now and ends when it has caught up.
- Python surface:
  - `discover()`, and `parse_line(line)`, which returns a dict or `None`.
  - `LogTail(path, from_start=False)`, whose `next_line()` returns a dict or `None`.
  - Iterating over a `LogTail` blocks until a line arrives, polling every 250 ms with the GIL released.
- Names follow ADR 0001:
  - on npm, `poe2-core`, with `"private": true` until publishing is decided;
  - on PyPI, `poe2-core`, with the module `poe2_core`. The Python crate's library is named `poe2_core`, so it renames its dependency on the core to `poe2`.
- Tooling stays local: `bindings/node/node_modules` holds `@napi-rs/cli`, and `bindings/python/.venv` holds maturin. Nothing is published.

**Considered options.**
- Bindings as members of the root workspace: the root test run would then have to link against Node and libpython.
- `#[napi(object)]` and `#[pyclass]` for each type: more code, and a second shape to freeze.
- napi's iterator support: errors couldn't propagate through it, so a three-line JS wrapper adds `Symbol.iterator` over `nextLine()` instead.

**Consequences.** The bindings stay thin and follow the JSON shape automatically. The JSON round trip costs a little for each line, which doesn't matter at the rate a log grows.
