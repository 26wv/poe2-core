# Locale tables beyond English

**Context.** v1 matches system messages and whisper markers through an English-only locale table (ADR 0009). The research found these samples ([client-log.md](../research/client-log.md) "Localisation"):
- Korean level-up, slain and AFK lines (poe2-saegida, single source);
- Japanese zone entered and level-up (poenavi, trimmed fixtures without the `: `);
- German zone entered (Exile-UI, PoE1 era).

These questions are still open:
- whether `@From` and `@To` are translated;
- whether Japanese lines keep the `: `;
- which languages the client offers.

**Needed.**
- For each language, anonymised real lines for every locale-dependent v1 event.
- A decision on how callers pick a locale: an explicit choice, or detection (poe2-saegida detects the language from `[SCENE]` area names).

**Acceptance criteria.**
- Each new table has corpus lines and snapshots for every event it covers.
- The public way to choose a locale is recorded in an ADR.
- English output is unchanged.
