# ADR 0019: Bottles read errors go on the candidates they affect

Status: accepted, 2026-10-24. Applies ADR 0015 to Bottles.

**Context.** ADR 0015 adds a launcher file's read error (`; <path>: <error>`) to its root's note, and the root is a fixed location listed as a would-be candidate. Bottles has no fixed location (ADR 0015), so its `data.yml` and `placeholder.yml` errors have no root row to go on.

**Decision.**
- A `<base>/data.yml` read error is added to the note of every bottle found in that base's `bottles/` folder.
- A `placeholder.yml` read error is added to that bottle's note, and the bottle's own folder is probed as if it had no placeholder.
- When nothing is found in a base, its read errors are not reported. No row is invented for them.

**Considered options.** A would-be candidate per Bottles base: it would list a row for an absent launcher on every Linux machine and contradict ADR 0015's "only when something is found".

**Consequences.** An unreadable `data.yml` hides the custom bottles folder. That shows up only when the same base has another bottle with PoE2 in it.
