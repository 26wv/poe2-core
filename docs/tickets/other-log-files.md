# Log files other than Client.txt

**Context.** v1 lists only `logs/Client.txt` in each install. The research found two other files ([client-log.md](../research/client-log.md)):
- `logs/KakaoClient.txt`, the log of the Korean (Kakao) client, for example under `C:\Daum Games\Path of Exile2\`. Two tools confirm it, but v1 has no install source for that client.
- `logs/LatestClient.txt`. Exile-UI tails it when it exists, and it looks like a current-session-only file, but nothing shows that PoE2 writes it (Conflicting 7, Unknown 7).

**Needed.**
- Whether PoE2 writes `LatestClient.txt`, and what it holds.
- A decision on whether the Kakao client is in scope. It needs an install source, and its Korean system messages need the `locale-tables` ticket.

**Acceptance criteria.**
- client-log.md is updated.
- Any file taken into scope becomes a client-log candidate with tests, and ranking still puts the newest log first.
