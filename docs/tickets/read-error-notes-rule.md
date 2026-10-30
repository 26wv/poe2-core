# One rule for where launcher read errors are noted

**Context.** Found in the v0.1.0 code review. Lutris (lutris.rs:28-65) drops a read error when a `games/*.yml` records the installer's default prefix, the common case. Steam (steam.rs:199-210) drops manifest errors whenever any library holds PoE2. Bottles puts errors on every candidate (ADR 0019). The review also found that Heroic (heroic.rs:49-70) still probes the default prefix twice when a GamesConfig `winePrefix` names `~/Games/Heroic/Prefixes/Path of Exile 2`, and the de-dup keeps the GamesConfig row, so the `probe_default` row's errors (heroic.rs:84, for example a config.json read error) are lost; the read-error handling at heroic.rs:74-83 covers only the `Prefixes/*` loop.

**Problem.** The four searches follow three different note rules, so whether a user sees "this file could not be read" depends on the launcher.

**Acceptance criteria.**
- A new ADR picks one rule; the consistent choice is ADR 0019's (errors on every candidate the search emits). Or ADR 0015's Consequences documents the gap.
- Lutris and Steam follow the chosen rule, with a test each.
- Heroic probes a GamesConfig prefix that equals the default only once (or merges the notes), so the default row keeps its errors; a test with such a GamesConfig and an unreadable config.json shows `list_count == 1` and the config.json note.
- ADR 0019's Consequences says the placeholder.yml error never surfaces in practice (or the test at bottles.rs:203-210 shows when it does).

**Rank.** should
