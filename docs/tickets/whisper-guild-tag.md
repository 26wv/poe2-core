# Guild tags in whisper lines

**Context.** PoE1 can put a guild tag before the name: `@From <<tag>> <name>: <text>`, and the same with `@To` (poe-log-monitor, in [client-log.md](../research/client-log.md)). No PoE2 source shows this. v1 takes everything between `@From ` or `@To ` and the first `: ` as the name, so a tag would end up inside the name (ADR 0007).

**Needed.** Whether PoE2 writes guild tags in whispers, with one anonymised line if it does.

**Acceptance criteria.**
- If tags exist: corpus lines and snapshots, and the whisper events expose the bare name with the tag in a separate field.
- If not: client-log.md says so and this ticket is closed.
