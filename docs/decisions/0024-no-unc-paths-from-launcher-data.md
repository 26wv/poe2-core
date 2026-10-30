# ADR 0024: No UNC or device paths from launcher data

Status: accepted, 2026-10-24. Amends ADR 0020.

**Context.** ADR 0020 accepted a Windows launcher path when it started with `X:\` or was a UNC path (`\\…`). Those paths come from text other programs and users can write: the Steam and GGG registry values, Steam's `libraryfolders.vdf` and Epic's `.item` manifests. Steam's folder and `ProgramData` are often writable by every local user. Probing `\\host\share\…` makes Windows open an SMB (or WebDAV) connection to that host and authenticate with the user's NTLM credentials, so a planted path can harvest another user's hash. `\\.\pipe\…` connects to a named pipe whose server can impersonate the caller or stall the read, and `\\.\…` and `\\?\…` reach devices and raw object paths. A passive reader must not make network or device connections because a file said so.

**Decision.**
- `tpath::absolute`, the filter for every untrusted Windows path, accepts only `X:\…` after normalising. Every path that starts with `\\` (UNC, `\\?\…`, `\\?\UNC\…`, `\\.\…`) is skipped like any other unacceptable value, silently and without a note (ADR 0020). Forward slashes are normalised first, so `//host/share` is caught too.
- Values the user or the OS sets may still be UNC: the `POE2_INSTALL_DIR` and `POE2_USER_DIR` overrides, the other env values the locator reads as paths, and the Documents known folder (a redirected Documents folder on a share is common).

**Considered options.**
- Keep UNC and only drop `\\.\` and `\\?\` forms: a plain `\\host\share` path is the credential leak.
- Allow UNC when it names the same host as the Documents folder: more code for a rare layout, and the host is still chosen by a file.

**Consequences.** A Steam library or Epic install genuinely on a network share is no longer found from launcher data; such a user sets `POE2_INSTALL_DIR`. A `\\?\D:\…` long-path value is skipped too, which costs at most a miss.
