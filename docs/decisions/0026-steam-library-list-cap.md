# ADR 0026: A Steam root takes at most 64 libraries from its library lists

Status: accepted, 2026-10-24. Applies ADR 0015's note rule to a new limit.

**Context.** Probing a library lists its folder (to find `steamapps` in any case), so the work grows with the number of libraries times the folder's size. Repeated libraries are dropped by their normalised key, but `key` folds only case (on Windows), slash style and trailing separators. `/x//big`, `/x/./big` and the like stay distinct, and a 1 MiB `libraryfolders.vdf` can hold tens of thousands of them, all pointing at one big folder. That costs minutes of CPU, which SECURITY.md counts as a hang. Real installs have a handful of libraries.

**Decision.**
- After de-duplication, a Steam root keeps its own folder plus the first 64 libraries from its two library lists, in list order. The rest are dropped.
- When entries are dropped, `; library list truncated to 64 entries` is added to the root's notes the way read errors are (ADR 0015). It shows on the root's would-be row when no install is found there.

**Considered options.**
- Normalising `.`, `..` and repeated separators in `key`: it narrows the spellings but not the count, and `..` can't be folded without resolving links.
- Probing `exists(<library>/steamapps)` before listing the folder: cheaper per entry, but the count is still unbounded.

**Consequences.** Work per root is bounded at 65 folder listings, whatever the file says. Someone with more than 64 libraries loses the ones past the cap, and the note says so; they can set `POE2_INSTALL_DIR`. As with read errors, the note is not shown when the root does have an install.
