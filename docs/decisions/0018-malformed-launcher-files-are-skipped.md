# ADR 0018: Malformed launcher files are skipped without a note

Status: accepted, 2026-10-24. Amends ADR 0005.

**Context.** ADR 0005 says a launcher file reader that meets malformed input returns "not found" with a note. The original readers instead return nothing for a malformed file: `library_paths` gives an empty list and `installdir` gives `None`, and the same holds for the JSON and YAML readers. ADR 0015 already notes files that can't be read (`; <path>: <error>`), but it has no note for a file that was read and turned out to be malformed.

**Decision.** In v1, malformed launcher files are skipped without a note. Read errors, including files over the 1 MiB read cap, are noted as ADR 0015 says. A malformed file never stops discovery.

**Considered options.** We could give each reader a `Result` and add a `malformed` note. That would mean a second error path in every reader and a new note text, all for files Steam and the launchers write themselves, which are rarely malformed.

**Consequences.** A corrupt library list or app manifest shows up only through its root's would-be candidate (ADR 0015), with no note saying why. If users report silent misses, reconsider adding the note.
