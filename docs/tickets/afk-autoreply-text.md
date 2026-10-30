# AFK autoreply text in English PoE2 lines

**Context.** The PoE2 sources match only the stems `: AFK mode is now ON` and `: AFK mode is now OFF`. PoE1 adds `. Autoreply "<text>"` to the ON line, and PoE2's Korean ON line includes the autoreply text too ([client-log.md](../research/client-log.md)). v1 accepts the PoE1 shape as an optional `autoreply` on `AfkOn`, and any other suffix makes the line `Unknown` (ADR 0007).

**Needed.** Anonymised English PoE2 AFK-on lines, with the default autoreply and with a custom one, plus an AFK-off line.

**Acceptance criteria.**
- The lines are in `tests/corpus/` with snapshots.
- `AfkOn` matching follows the real shape.
- The AFK rows in client-log.md are updated.
