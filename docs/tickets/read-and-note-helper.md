# Share the read-and-note and XDG helpers across the launcher searches

**Context.** Found in the v0.1.0 code review. The block "read the file, skip `NotFound`, append `; {file}: {e}`" is written four times (steam.rs:153-168, :194-201; heroic.rs:50-57; lutris.rs:35-42), besides `bottles::read` (bottles.rs:71-80), which already is the helper. The XDG fallback is written four times (steam.rs:59-62, heroic.rs:21-24, lutris.rs:18-22, bottles.rs:22-25) and the absolute-path rule for launcher values is hand-written as `starts_with('/')` seven times. Smaller repeats: steam.rs:157-160 re-reads `home_dir()` per library; the `C:\Program Files (x86)` default and `env_or` sit in steam.rs:33-35 and windows.rs:19, :31; the test constant `GGG` copies `prefix::INSTALL` four times; the `My Games/Path of Exile 2` tail is written three times.

**Problem.** One rule lives in up to nine places, so a fix to one copy can easily miss the others. Steam's own `read()` copies the whole file only to strip a BOM.

**Acceptance criteria.**
- One `read_noted` helper (moved from `bottles::read`) is used by all four searches; `steam::read` is gone and `keyvalues::parse` strips the BOM, as json.rs and yaml.rs do.
- One `xdg(sys, home, name, default)` helper; every untrusted launcher path goes through `tpath::absolute`, and the "never resolved against the cwd" comment appears once, on the helper.
- The no-HOME early returns in heroic.rs, lutris.rs, bottles.rs and prefix.rs share one `Known limit:` comment (only steam.rs:54 has one now).
- `home` is read once in `search_linux`; one `env_or` and one Program Files default; tests use `prefix::INSTALL`; one `USER_DIR` constant.
- All locate tests still pass; no public API change (ADR 0022).

**Rank.** should
