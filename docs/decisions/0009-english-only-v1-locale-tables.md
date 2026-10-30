# ADR 0009: English-only v1, with localised messages matched through a locale table

Status: accepted, 2026-10-23

**Context.** Every `: ` system message (zone entered, level-up, slain, AFK) is translated, and so are class and area names. Word order and punctuation change too: Korean puts the level before the verb and drops the space before `(`, and Japanese uses different brackets and full stops ([client-log.md](../research/client-log.md) "Localisation"). Engine lines stay English in every client: the prefix, `***** LOG FILE OPENING *****`, `Generating level …` and the `[SCENE] Set Source [` wrapper. Nobody knows whether `@From` and `@To` are translated. The file name doesn't reveal the language, because the Kakao client can be switched to English. The only samples are Korean (single source), Japanese (trimmed fixtures) and PoE1-era German.

**Decision.**
- v1 recognises English system messages only.
  - Non-English system messages become `Event::Unknown`, which loses nothing (ADR 0008).
  - The engine-line events (`LogOpened`, `AreaGenerated`, `SceneSet`) work in every client.
- All locale-dependent text lives in one place, a **locale table**.
  - It covers `ZoneEntered`, `LevelUp`, `CharacterSlain`, `AfkOn`, `AfkOff` and the whisper markers `@From` and `@To`.
  - Each entry is one or more templates of literal text with named placeholders, such as `{name} ({class}) is now level {level}`, so another locale can reorder the placeholders.
  - Matching code reads only the table. v1 ships one table, English, and `parse_line` uses it.
  - Engine lines are matched outside the table.
- v1 has no language detection and no public way to choose a locale. Adding a table is an internal change, and a public way to choose one is an additive change later.
- Template matching is plain string work, with no regex crate:
  - literal segments anchor the match;
  - placeholders capture what lies between the literal segments;
  - typed placeholders, such as levels, must parse.

**Considered options.**
- Hard-coded English matchers: the fastest to write, but adding Korean would mean rewriting each one.
- The `regex` crate: several crates of dependency for patterns this simple. Templates can still be compiled to regexes later.
- Shipping Korean and Japanese now: the evidence is single-source or comes from trimmed fixtures.

**Consequences.** Non-English players get engine events plus `Unknown` lines in v1, and the README must say so. The ticket `locale-tables` lists the samples and the open questions.
