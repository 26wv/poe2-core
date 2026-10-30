# ADR 0021: An AFK-on line with an empty autoreply stays Unknown

Status: accepted, 2026-10-24

**Context.** The English `afk_on` templates carry an optional `. Autoreply "{autoreply}"` suffix (ADR 0007, ticket `afk-autoreply-text`). The template matcher rejects empty captures except `{text}` (the doc comment on `match_template` in `crates/poe2-core/src/log/locale.rs`), and a body that doesn't fit a template is `Unknown` (ADR 0007 "Matching rules"), so `: AFK mode is now ON. Autoreply ""` matches no template. No source shows such a line in PoE2, and the English locale table has no template for it.

**Decision.** That line stays `Event::Unknown`. The matcher keeps its single rule for empty captures, and nothing special-cases `{autoreply}`.

**Considered options.**
- `AfkOn { autoreply: Some("") }`: needs an exception in the matcher for a line nobody has seen.
- `AfkOn { autoreply: None }`: would hide that the line had a suffix.

**Consequences.** If a real PoE2 line shows an empty autoreply, the ticket `afk-autoreply-text` revisits this; `Unknown` keeps the text meanwhile (ADR 0008).
