# ADR 0020: Windows launcher values must be absolute; unreadable Epic manifests are skipped

Status: accepted, 2026-10-24. Applies ADRs 0015 and 0018 to the Windows search.

Amended by [ADR 0024](0024-no-unc-paths-from-launcher-data.md): UNC and other `\\…` paths are skipped too.

**Context.** The standalone registry value and an Epic manifest's `InstallLocation` are untrusted text. The Heroic, Lutris, Bottles, standalone and Epic searches skip an empty or relative launcher path instead of resolving it against the working directory, but "absolute" needs a Windows meaning. ADR 0015 notes a launcher file's read error on its root's row, and Epic has no fixed location (ADR 0015), so an unreadable `.item` file has no row to carry the note.

**Decision.**
- A Windows path from the registry or a manifest is used only when, after normalising, it starts with a drive and a backslash (`X:\…`) or is a UNC path (`\\…`). Empty values, `X:` alone, `\…` and relative paths are skipped. The registry value gets the same rule as manifest values.
- An `.item` file that can't be read (including one over the 1 MiB read cap) is skipped without a note, as ADR 0019 does for a Bottles base with nothing found. No row is invented for it.

**Considered options.** Adding an unreadable manifest's error to every Epic candidate's note: one manifest describes one game, so the error says nothing about the others.

**Consequences.** An unreadable PoE2 manifest is a silent miss. If users report one, add a would-be Epic row or a `checked` note.
