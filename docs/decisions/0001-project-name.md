# ADR 0001: Keep the name poe2-core

Status: accepted, 2026-10-23

**Context.** The project is called `poe2-core`, and the name would change if it was taken on crates.io, npm or PyPI. On 2026-10-23 every checked variant returned 404 on all three registries, and positive controls returned 200 ([name-availability.md](../research/name-availability.md)).

**Decision.** Keep the names. The library crate, the npm package and the PyPI project are `poe2-core` (the Rust and Python import name is `poe2_core`). The CLI crate is `poe2-cli`, and it installs the binary `poe2`.

**Consequences.** A name is held only once something is published under it. Re-run the checks before each package's first publish. If a name has been taken by then, a new ADR supersedes this one.
