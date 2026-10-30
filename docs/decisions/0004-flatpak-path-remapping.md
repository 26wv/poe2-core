# ADR 0004: Remap paths recorded inside Flatpak sandboxes

Status: accepted, 2026-10-23

**Context.** Flatpak Steam (`com.valvesoftware.Steam`) records sandbox paths. Library `"0"` can read `$HOME/.local/share/Steam` (or the full Flatpak path) while the folder is `~/.var/app/com.valvesoftware.Steam/.local/share/Steam` on the host. The sandbox's own `~/.steam/root` and `~/.steam/steam` links hold sandbox targets, and on the host those can resolve to a native Steam install ([linux-paths.md](../research/linux-paths.md) Q1, Conflicting). steamlocate follows those links and doesn't remap; protontricks remaps. The research infers the same pattern, untested, for the Heroic Flatpak (`--persist=.`) and for Bottles, which can record `/var/data/…`.

**Decision.**
- Probe Flatpak roots directly: `~/.var/app/com.valvesoftware.Steam/.local/share/Steam` and the legacy `…/data/Steam`. Never resolve a sandbox's `.steam/*` links on the host.
- A path read from a Flatpak app's own files yields up to two candidates: the path as written, and the path remapped into the app's host folder. Remapping turns `$HOME/<rel>` into `~/.var/app/<app-id>/<rel>`, and `/var/data/<rel>` into `~/.var/app/<app-id>/data/<rel>`. Paths under `/mnt`, `/media` and `/run/media` are already host paths and are not remapped. Ranking puts the candidate that exists first.
- Canonicalise host paths only, never a sandbox path before it has been remapped.
- Snap Steam roots (`~/snap/steam/common/.local/share/Steam`, `~/.snap/data/steam/common/.local/share/Steam`) are probed directly with no remap. Snap changes only `$HOME`.

**Consequences.** Both forms of library `"0"` work, at the cost of a few extra probes. The inferred Heroic and Bottles cases are covered without trusting the inference. The ticket `linux-launcher-layouts` checks them on real installs, and any remap that proves unnecessary is removed.
