# Suicide message

**Context.** Exile-UI lists `: <name> has committed suicide.` for "PoE1 and PoE2", and no other source has it ([client-log.md](../research/client-log.md) Unknown 9). v1 leaves the line `Unknown`, so a death counter built on `CharacterSlain` misses suicides.

**Needed.** One anonymised English PoE2 line.

**Acceptance criteria.**
- The line is in `tests/corpus/` with a snapshot.
- It is matched through the locale table, either as a new event or as a field on `CharacterSlain`.
