# ADR 0011: Locator tests are unit tests, and `System` stays crate-private

Status: accepted, 2026-10-23

**Context.** ADR 0002 makes `FakeSystem` test-only and crate-private, and a public `discover_with(sys: &impl System)` would be the natural test entry point. A public function bounded by a crate-private trait trips the `private_bounds` lint, so either the trait becomes public or `discover_with` becomes private. A public `System` would freeze ten methods that outside code would have to implement. Nothing outside the crate needs it in v1, because the CLI and the bindings call `discover()`.

**Decision.**
- `System`, `TargetOs`, `Hive`, `discover_with`, the real system and `FakeSystem` are `pub(crate)`. The public locator API is `discover()`, `Discovery`, `Found` and `Source`.
- Locator tests are unit tests inside `crates/poe2-core/src/locate/` (`#[cfg(test)]`), where `FakeSystem` and `discover_with` are in reach. Each search module tests its own search function through a test-only `collect(sys, search)` helper. One test in `locate.rs` checks that every `Source` appears.
- The real system's file operations are tested against temporary folders on the host. Its registry and known-folder wrappers are compile-checked on Linux with `--target x86_64-pc-windows-msvc`, and they run on Windows CI.
- Tests outside the crate (CLI, bindings) run the real system only with `HOME` pointed at an empty temporary folder.

**Considered options.**
- A `test-support` cargo feature that makes `FakeSystem` public: it would be public API by another door, plus a feature combination to test.
- `#[doc(hidden)] pub`: the same, but undocumented.
- A public `System` now: it could let users probe a mounted Windows disk from Linux, but nobody has asked for that, and it freezes the trait.

**Consequences.** Making `System`, `discover_with` and `FakeSystem` public later is an additive change. Until then, integration tests can't build fake layouts. On Linux, the CLI's end-to-end tests build real layouts under a temporary `HOME` instead.
