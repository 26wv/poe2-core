# Evidence for the whisper-sent line

**Context.** `@To <name>: <text>` is confirmed only thinly. mapwatch parses it, JuiceJournal mentions it in a comment, and the other evidence is PoE1. No PoE2 fixture contains one ([client-log.md](../research/client-log.md)). v1 includes `WhisperSent` anyway (ADR 0007).

**Needed.** One anonymised English PoE2 `@To` line from a real client.

**Acceptance criteria.**
- The line is in `tests/corpus/`, and its snapshot parses as `WhisperSent`.
- The client-log.md row no longer says "thin".
