# Log parser: one digits helper and test gaps

**Context.** Found in the v0.1.0 code review. Two copies of the `digits` helper (numbers are parsed only after an all-ASCII-digits check) exist (log/line.rs:171-177, log/event.rs:163-170); the event.rs copy lacks the empty check. Untested: a multi-byte char straddling byte 20 and a tab in the log source; multi-byte text on the final-literal and first-occurrence paths; an `ENGLISH` template with unbalanced braces (it would silently never match); a doubled prefix (`": : Generating"`), a non-DEBUG level and leading zeros; pid 4294967295 parsing (4294967296 is covered at line.rs:266) and the `level`/`area_level` bounds; `@To` negatives (look-alike, empty name, guild tag); an Unknown corpus line starting with a BOM. An empty-text `@From` parses as `WhisperReceived { text: "" }` with no assertion that this is deliberate. The first `UPDATE_SNAPSHOTS=1` run races the fixture-exists test (corpus.rs:94-109).

**Problem.** A safety helper has drifting copies, and several parser edges rely on construction rather than a test.

**Acceptance criteria.**
- line.rs's `digits` is `pub(super)` and event.rs uses it.
- A test asserts every locale template has balanced braces.
- Tests cover the edges above, with the empty `@From` text asserted explicitly; the snapshot race is fixed or documented.

**Rank.** should
