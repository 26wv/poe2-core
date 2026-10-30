# ADR 0008: Every non-blank line yields one LogLine, and unknown lines keep their text

Status: accepted, 2026-10-23

**Context.** A line that matches none of ADR 0007's events must become `Event::Unknown { raw }`: a line is never dropped, and unknown lines round-trip. The signature is `parse_line(&str) -> Option<LogLine>`. Not every line carries the standard prefix ([client-log.md](../research/client-log.md) Unknown 4, 5, 11):
- the log-opened marker has only a date and a time;
- a BOM can precede the first line after a tool recreates the file;
- blank lines occur (three tools skip them);
- some tools' tests feed unprefixed lines;
- patches add and remove messages.

**Decision.** `parse_line` works as follows.
1. The input is one line. `parse_line` removes one trailing `\n`, then one trailing `\r`, then one leading U+FEFF. What remains is the line text.
2. Empty line text gives `None`, and this is the only case that does. A blank or BOM-only line carries nothing.
3. Any other line text gives `Some(LogLine)`:
   - A line that matches the full prefix grammar gets its prefix fields (date, time, ms counter, hex id, log level, log source, pid) plus the event its body matches, or `Event::Unknown`.
   - `<date> <time> ***** LOG FILE OPENING *****` gives `Event::LogOpened`, with the date and time and no other prefix fields.
   - Anything else has no prefix fields and gives `Event::Unknown`.
4. `Event::Unknown { raw }` holds the entire line text from step 1, prefix included, so `raw` reproduces the line exactly whether or not it had a prefix. The prefix of an unknown prefixed line is still parsed, for consumers who want its time or pid.
5. Callers don't filter. `LogTail` and `poe2 parse` yield one `LogLine` per non-blank line, in file order.
6. Round-tripping is defined on decoded text:
   - Whatever reads the file decodes it as lossy UTF-8 before `parse_line`, so invalid bytes become U+FFFD.
   - A line longer than LogTail's cap is cut at the cap (ADR 0010), and its `raw` is the part that was kept. The game doesn't write lines anywhere near that long.
7. `parse_line` never panics on any `&str`, and a property test backs this.

**Considered options.**
- `None` for every unprefixed line: callers would have to re-wrap such lines or lose them, which the rule above forbids.
- `raw` as the body only: round-tripping would depend on re-formatting the prefix exactly, and unprefixed lines would need a different rule.
- Returning `LogLine` without `Option`: it changes the signature above, and blank lines would need a dummy event.

**Consequences.**
- A snapshot of an unknown line shows its full text, which is what the "Unparsed log line" issue template asks users for.
- Consumers tell "known event" from "line we don't understand" with one match arm.
- Because `raw` carries whole lines, every corpus line must be synthetic.
