# Locator test gaps

**Context.** Found in the v0.1.0 code review.
- heroic.rs:199-216 `empty_and_relative_prefixes_are_skipped` can't fail: with the filter at heroic.rs:61 removed, all Heroic tests stay green (mutation run). Lutris (lutris.rs:139-150) and Bottles (bottles.rs:157-177) plant sentinel folders; Heroic doesn't.
- Five `discover_with_runs_*` tests (lutris.rs:204-209, bottles.rs:224-228, heroic.rs:258-263, steam.rs:440-448, :792-803) only repeat `every_source_is_reachable`, whose Source list (locate.rs:204-216) is hand-written.
- Untested branches: Steam's third registry key, two-root order, config library-list read error, empty-registry-root guard; `is_shortcut_id` bounds, plain compatdata note, non-x86 Program Files, real My Documents, lowercase Public, compatdata under Flatpak; user dir/build_planner/filters ranking when only some exist; Bottles running before `search_wine`; a manifest named just `.item` is accepted.

**Problem.** A trust-boundary filter has no test that bites, and new `Source` variants are not forced into the reachability test.

**Acceptance criteria.**
- The Heroic test plants `/{UD}` and `/rel/poe2/{UD}` and fails with the filter removed.
- The Source list is built from an exhaustive in-crate `match`, so a new variant fails to compile; the five redundant tests are deleted.
- Tests cover the branches above, or the ticket records why one is dropped.
- A manifest named `.item` is rejected or the acceptance is documented.

**Rank.** should
