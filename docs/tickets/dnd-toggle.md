# DND toggle event

**Context.** PoE1 writes `: DND mode is now ON. Autoreply "<text>"` and `: DND mode is now OFF` (poe-log-monitor, in [client-log.md](../research/client-log.md)). No PoE2 source shows either line, so v1 parses both as `Event::Unknown` (ADR 0007).

**Needed.** Anonymised English PoE2 lines from a real client for DND on and for DND off.

**Acceptance criteria.**
- The lines are in `tests/corpus/` with snapshots.
- `Event` gains DND variants (or a shared away-status event with AFK), matched through the English locale table.
- The CHANGELOG notes that these lines are no longer `Unknown`.
