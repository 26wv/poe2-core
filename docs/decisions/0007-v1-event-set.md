# ADR 0007: The v1 event set

Status: accepted, 2026-10-23

**Context.** An event gets its own type only if there is real evidence of its format. [client-log.md](../research/client-log.md) gives each event a status.
- CONFIRMED: log opened, area generated, scene set, zone entered, level-up, character slain, whisper received, whisper sent (thin evidence), and AFK on and off (the stem only).
- UNCONFIRMED: DND on and off, and guild tags in whispers (PoE1 evidence only).

The zone line `: You have entered <area name>.` is localised and can be missing. One tool says Local chat must be enabled, and a measured 0.5.5e excerpt shows a hideout load without the line. `[SCENE] Set Source [<area name>]` is an engine line with an English wrapper, and it appears on every load in the evidence.

**Decision.** `Event` is `#[non_exhaustive]` and has exactly these v1 variants. The implementation chooses the field types.

| Variant | Body after the prefix | Fields |
|---|---|---|
| `LogOpened` | the whole line is `<date> <time> ***** LOG FILE OPENING *****` | none beyond the line's date and time |
| `AreaGenerated` | `Generating level <lvl> area "<area id>" with seed <seed>` | area level, area id (opaque, as written), seed (u64) |
| `SceneSet` | `[SCENE] Set Source [<area name>]` | area name, verbatim, including `(null)` and `(unknown)` |
| `ZoneEntered` | `: You have entered <area name>.` | area name, with one final `.` removed |
| `LevelUp` | `: <name> (<class>) is now level <n>` | character name, class (can be several words), character level |
| `CharacterSlain` | `: <name> has been slain.` | character name |
| `WhisperReceived` | `@From <name>: <text>` | sender, text |
| `WhisperSent` | `@To <name>: <text>` | recipient, text |
| `AfkOn` | `: AFK mode is now ON`, then optionally `. Autoreply "<text>"`, then optionally `.` | autoreply text, if present |
| `AfkOff` | `: AFK mode is now OFF`, then optionally `.` | none |
| `Unknown` | anything else (ADR 0008) | `raw` |

**Zone changes.** The parser neither merges nor de-duplicates `SceneSet` and `ZoneEntered`: each line yields its own event, and no consumer needs both. Consumers that want a reliable signal of a zone change use `SceneSet` and skip its `(null)` (loading) and `(unknown)` (login, character select) names. `ZoneEntered` is the game's own localised message and may be absent.

**Matching rules**, all from the research:
- Match at the start of the body, never by substring. Trade chat can contain ` : <name> (<class>) is now level <n>`. System messages start with `: `, and chat lines never do.
- Don't key events on the log level. `[SCENE]` and `Generating level` lines appear as both INFO and DEBUG, and `Generating level` may come after a stray `: `.
- Names are not limited to ASCII (Kakao names can be Hangul).
  - A level-up or slain name runs up to its first space.
  - A whisper name runs up to the first `: `, because the text can contain `: `. If PoE2 writes guild tags, the tag therefore stays inside the name (ticket `whisper-guild-tag`).
- A body that starts like an event but doesn't fit it (a bad number, an unexpected suffix) is `Unknown`, never a half-filled event.
- Level-ups and deaths of party members carry those members' names. The consumer decides whose they are.
- `AfkOn` accepts the PoE1-shaped autoreply suffix as an optional field, because PoE2's Korean ON line also carries the autoreply text (ticket `afk-autoreply-text`).
- DND lines and `has committed suicide.` stay `Unknown` in v1 (tickets `dnd-toggle`, `suicide-message`).

**Consequences.**
- Every variant needs a corpus fixture and a snapshot.
- A new event later is an additive change to the type. It does change some lines from `Unknown` to typed, so it gets a CHANGELOG entry.
- Marking struct-like variants `#[non_exhaustive]` lets later tickets add fields, such as a guild tag.
- The locale-dependent variants (`ZoneEntered`, `LevelUp`, `CharacterSlain`, `AfkOn`, `AfkOff` and the whisper markers) are matched through the locale table (ADR 0009).
