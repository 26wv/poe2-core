# Confirm the standalone install's registry key

**Context.** v1 reads `HKCU\Software\GrindingGearGames\Path of Exile 2` → `InstallLocation` as the standalone install's location. Only one forum post names this key, though TraXile reads the same pattern for PoE1 ([windows-paths.md](../research/windows-paths.md) Q3).

These are unknown:
- whether the installer or the game writes the value;
- whether it has a trailing `\`;
- whether a `WOW6432Node` or HKLM copy exists;
- the Uninstall entry. None was found, and winget has no GGG manifest.

v1 also lists `C:\Program Files (x86)\Grinding Gear Games\Path of Exile 2` and the same under `C:\Program Files\` as fallbacks.

**Needed.** On a real Windows machine with the standalone client, the output of these commands, with user names in paths anonymised:
- `reg query "HKCU\Software\GrindingGearGames" /s`, and the same under `HKLM\SOFTWARE` and `HKLM\SOFTWARE\WOW6432Node`;
- `reg query HKCU\Software\Microsoft\Windows\CurrentVersion\Uninstall /s /f "Path of Exile 2"`, and the HKLM and `WOW6432Node` variants.

**Acceptance criteria.**
- windows-paths.md records the findings.
- The locator reads every confirmed key, with `FakeSystem` tests.
- A trailing `\` in the value is handled.
