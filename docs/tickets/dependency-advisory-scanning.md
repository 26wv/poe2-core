# Scan dependencies for advisories

**Context.** Found in the v0.1.0 code review. There is no `cargo audit`/`cargo-deny`, no `npm audit` and no Dependabot for the three Cargo.lock files and bindings/node/package-lock.json. No advisory scan has been run yet. Lockfile sources are all crates.io or registry.npmjs.org with checksums, with no git deps or install scripts.

**Problem.** A known-vulnerable crate or npm package would go unnoticed.

**Acceptance criteria.**
- A CI job runs `cargo deny check advisories bans sources` (or a SHA-pinned audit action) over all three lockfiles.
- npm advisories are checked (`npm audit` or Dependabot).
- Dependabot covers `cargo` and `npm`.

**Rank.** should
