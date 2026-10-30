# ADR 0010: LogTail polls the path and treats a shrink as a new file

Status: accepted, 2026-10-23

**Context.** Client.txt is append-only and grows to hundreds of MB. No source reports the game rotating or truncating it, but tools do: Exile-UI's backup recreates the file, possibly with a BOM. Nobody knows what the game does if the file is cut while it runs, which Linux allows ([client-log.md](../research/client-log.md) "File properties", Unknown 4 and 6). JuiceJournal treats a shrink as rotation and restarts from 0. The tail must poll (no file watcher unless polling provably fails), start from the end by default, handle CRLF, invalid UTF-8 and partial lines, and use bounded memory.

**Decision.**
- **Follow the path, not a handle.** Each poll opens the path, compares its length with the read position, and reads only the new bytes, in bounded chunks. A handle kept open on Linux would go on reading a deleted file forever.
- **Shrinking.** A length below the read position is a truncation: the read position returns to 0. A held partial line is first yielded as it stands, so no text is lost.
- **Starting point.** By default reading starts at the current end of the file. If the file doesn't end in `\n`, everything up to the next `\n` is skipped, so the first line yielded is whole. `--from-start` and `poe2 parse` start at 0.
- **Line ends.** A line is complete only at `\n`. The bytes after the last `\n` are a partial line, held until the rest arrives. An offline read (`poe2 parse`) yields an unterminated last line at end of file.
- **Decoding.** Each complete line is decoded as lossy UTF-8 and passed to `parse_line`, which removes `\r` and a leading BOM (ADR 0008).
- **Long lines.** A line longer than a fixed cap is cut at the cap. The implementation picks the cap; about 64 KiB is far above anything the game writes. The kept part is yielded as a line, and the rest is skipped up to the next `\n`.
- **Real files.** LogTail uses `std::fs` directly, and its tests use temporary files. `System` is only for the locator.

**Considered options.**
- File watchers such as `notify`: a dependency, and v1 polls first.
- Tracking file identity (inode or file index): not portable in std. The shrink check catches every rotation the evidence suggests, because a new file starts small.
- Keeping one handle open: it misses a delete followed by a recreate on Linux.

**Consequences.**
- Memory per tail is bounded by the line cap plus one read chunk.
- One case is accepted as a known gap: if the file is replaced by a longer file between two polls, it is read from the old position. The game only recreates the file at launch, when it is small.
