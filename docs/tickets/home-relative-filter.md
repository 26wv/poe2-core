# Treat a relative HOME as unset

**Context.** Found in the v0.1.0 code review. ADR 0025 makes relative env path values count as unset, but `RealSystem::home_dir` (crates/poe2-core/src/locate/real.rs:28-29) returns `std::env::home_dir()` unfiltered. Only the Linux searches call it (steam, heroic, lutris, bottles, prefix). ADR 0025's Consequences already records the gap.

**Problem.** A relative `HOME` makes those searches probe paths under the working directory, the class ADR 0025 closed for every other env path.

**Acceptance criteria.**
- `home_dir` keeps only absolute values (`.filter(|h| h.starts_with('/'))` at real.rs:29), so a relative HOME reads as no HOME.
- A test with a relative HOME lists no row outside the real home (or a unit test of the filter if setting HOME in-process is unsafe).
- ADR 0025's Consequences no longer lists HOME as a gap.

**Rank.** nice
