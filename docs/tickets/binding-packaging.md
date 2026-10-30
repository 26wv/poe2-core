# Binding and crate packaging gaps before publishing

**Context.** Found in the v0.1.0 code review. Node: no LICENSE files or README in the package, no `"types"` field (`main: lib.js` hides index.d.ts from TypeScript), no `[Symbol.iterator]` declared on `LogTail`, no per-platform binary packages, and `"private": true`. Python: no licence texts, readme or project URLs in the sdist/wheel, the version lives in two places, no abi3 wheel. Crate: `cargo package --list` ships `src/locate/fixtures/` and `gen_yaml.py`.

**Problem.** The packages can't be published as they are: licences would be missing, TypeScript users get no types, and npm/PyPI users would need a compiler or one wheel per Python version.

**Acceptance criteria.**
- Node: LICENSE-MIT and LICENSE-APACHE plus a README ship; `"types"` points at index.d.ts and declares the iterator; per-platform npm packages (napi `optionalDependencies`) exist.
- Python: licence files, readme and URLs are in sdist and wheel metadata; one version source; abi3 wheels for the lowest supported Python.
- poe2-core's Cargo.toml `exclude`s the YAML fixtures and generator.
- The publish guards (`publish-guards` ticket, `"private": true`) are removed only as the deliberate last step before the first publish.

**Rank.** should
