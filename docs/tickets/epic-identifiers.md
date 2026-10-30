# Confirm PoE2's Epic identifiers and paths

**Context.** v1 finds Epic installs in `%PROGRAMDATA%\Epic\EpicGamesLauncher\Data\Manifests\*.item`, matched by `CatalogNamespace` `58a18be6bbba41a0bf52b014bb81d33b` (from Epic's own API) or by a `DisplayName` that starts with `Path of Exile 2` ([windows-paths.md](../research/windows-paths.md) Q4).

These claims have one source each, and v1 doesn't use them:
- `AppName` `57d39134358849b1b765ee649ea6a6e9` (main) and `54c5e100caf049bf88068afe4c02de21` (Free Weekend);
- `FolderName` `PathOfExile2` and the default root `C:\Program Files\Epic Games\`;
- the log at `C:\Program Files\Epic Games\PathOfExile2\logs\Client.txt` ([client-log.md](../research/client-log.md)).

These are unknown:
- `LaunchExecutable`;
- whether `DisplayName` is ever localised;
- the `LauncherInstalled.dat` schema.

On Linux, Heroic's legendary `installed.json` would give the install dir of a Heroic-Epic copy, but it is keyed by `AppName`.

**Needed.** An anonymised PoE2 `.item` file from a real Epic install, and if possible a Heroic `installed.json` entry for PoE2.

**Acceptance criteria.**
- windows-paths.md confirms or corrects each claim above.
- If they are confirmed: an Epic default-folder fallback, `LauncherInstalled.dat` as a second source, and Heroic-Epic installs from `installed.json`, each with `FakeSystem` tests.
