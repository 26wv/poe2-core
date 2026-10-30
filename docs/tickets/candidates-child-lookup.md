# candidates child(): exact join first, folders only

**Context.** Found in the v0.1.0 code review. `child()` (crates/poe2-core/src/locate/candidates.rs:113-126) lists and sorts the whole folder before matching, and matches any entry, so a file named like the wanted folder (for example `onlinefilters`) can win. The library de-dup at steam.rs:169-171 folds only trailing separators via `key`, so `/x//big` and `/x/./big` still count as separate libraries; ADR 0026 keeps at most 64 libraries per Steam root, which bounds the cost but still leaves duplicate rows and listings.

**Problem.** Every lookup costs a full listing of a possibly huge folder, and a file can stand in for a folder. Up to 64 spellings of one library each list it again and can each emit a row.

**Acceptance criteria.**
- `child()` tries `exists(join(dir, name))` before listing, and accepts only folders.
- Library paths are normalised before the de-dup (`//` folded, `/./` dropped; `..` left alone unless the path is canonicalised), so the steam.rs:629 test gains `/x//lib` and `/x/./lib` spellings that list once.

**Rank.** nice
