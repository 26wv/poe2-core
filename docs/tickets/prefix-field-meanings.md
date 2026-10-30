# Meaning of the prefix fields

**Context.** v1 parses the prefix as date, time, ms counter (u64), hex id (opaque string), log level, log source and pid ([client-log.md](../research/client-log.md) "Line prefix", Unknown 1–3 and 10). These meanings are single-source or unknown:
- that `<pid>` is the process id (poe2-saegida only);
- the counter's epoch, and whether it wraps at 2^32;
- what the hex id identifies;
- the time zone of the date and time (assumed local);
- whether other log levels or log sources exist.

**Needed.** Observations from a real client:
- the pid compared with the game's process id;
- the counter compared with uptime, across a reboot;
- timestamps compared with the system clock under a non-UTC time zone.

**Acceptance criteria.**
- client-log.md is updated.
- The rustdoc on the prefix fields states only confirmed meanings.
