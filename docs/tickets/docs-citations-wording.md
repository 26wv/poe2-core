# Fix stale ADR citations and wording

**Context.** Found in the v0.1.0 code review: small docs fixes.
- crates/poe2-core/src/log/locale.rs:48-49 credits the empty-capture rule to ADR 0007 "Matching rules", which lacks it; repoint to ADR 0021.
- ADR 0010:23 says "a longer file"; it should say "at least as long" (tail.rs:14 already does).
- ADR 0015's Source wording doesn't say ProtonPrefix wins for compatdata rows.
- ADR 0022: "patch or minor release" is ambiguous below 1.0; "exactly what this ADR lists" conflicts with the additive rule that allows new public items; it says the JSON examples come from tests/corpus, but the Discovery example is hand-written.
- ADR 0023:5's amendment note doesn't say that Node's `discover` and `parseLine` also stopped returning `serde_json::Value` through napi's `serde-json` feature.
- CONTRIBUTING's anonymising list names more sample names and paths than the issue template (a harmless superset).
- CONTRIBUTING's Python steps run `python3` in Git Bash on Windows, where it may not exist; mention `py` or `python`.
- docs/demo.tape has no `Sleep` before `Show` (cosmetic; the gif is not rendered).
- CHANGELOG.md has no `[Unreleased]` section or `[0.1.0]` compare link; add them once the repo URL exists.
- README's Node install block: its two `cd` lines don't chain (placeholders; readable as is).
- README's Wine prefix sentence (`drive_c/users/*/Documents/…`) doesn't say the `Public` user is skipped.
- README's Standalone and Epic matrix rows don't link their tickets (the CHANGELOG lists every ticket).
- README:225: the compatdata scan also falls under the 64-library cap; say so.
- "At most 64 libraries per root" is really the root folder plus 64 libraries.
- README:192-193: "absolute path" for the env vars could be read as excluding UNC, but `POE2_*` may be UNC.

**Problem.** Readers following a citation land on the wrong rule.

**Acceptance criteria.**
- Each item is fixed or recorded as no longer applying.
- No ADR's decision changes; wording only.

**Rank.** nice
