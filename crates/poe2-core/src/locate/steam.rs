//! Steam roots, libraries and app manifests on Windows (docs/research/windows-paths.md Q1–Q2, Conflicting 1 and 3) and Linux (docs/research/linux-paths.md Q1–Q4, Probe recipe 1–2, Conflicting), and Proton prefixes in each Linux library; ADRs 0002 §4–7, 0003, 0004, 0005, 0015, 0018.

use super::Source;
use super::candidates::{Candidates, child};
use super::keyvalues::{installdir, library_paths};
use super::prefix::probe;
use super::system::{Hive, System, TargetOs};
use super::tpath::{absolute, flatpak_remap, join, key, normalize};
use std::collections::HashSet;
use std::io::{self, ErrorKind};

/// PoE2's Steam store app ID.
pub(crate) const APP_ID: &str = "2694490";

/// Libraries taken from one Steam root's library lists, after de-duplication (ADR 0026).
const MAX_LIBRARIES: usize = 64;

/// Flatpak Steam's app ID.
const FLATPAK_ID: &str = "com.valvesoftware.Steam";

/// The registry's Steam roots, then the default Steam folder, each searched once.
pub(crate) fn search_windows(sys: &impl System, out: &mut Candidates) {
    let os = sys.target_os();
    let registry = [
        (Hive::CurrentUser, r"Software\Valve\Steam", "SteamPath"),
        (
            Hive::LocalMachine,
            r"SOFTWARE\WOW6432Node\Valve\Steam",
            "InstallPath",
        ),
        (Hive::LocalMachine, r"SOFTWARE\Valve\Steam", "InstallPath"),
    ];
    let program_files = sys
        .env_path("ProgramFiles(x86)")
        .unwrap_or_else(|| r"C:\Program Files (x86)".to_owned());
    let roots = registry
        .into_iter()
        .filter_map(|(hive, subkey, value)| sys.registry_string(hive, subkey, value))
        .filter_map(|root| absolute(os, &root))
        .map(|root| (root, Source::Steam))
        .chain([(join(os, &program_files, "Steam"), Source::Fallback)]);
    let mut seen = HashSet::new();
    for (root, source) in roots {
        if seen.insert(key(os, &root)) {
            search_root(sys, &root, source, out);
        }
    }
}

/// The native, Flatpak and Snap Steam roots in probe order, each searched once, then the Proton
/// prefixes in each of their libraries. The sandbox's own `.steam/*` links are never probed (ADR 0004).
pub(crate) fn search_linux(sys: &impl System, out: &mut Candidates) {
    let os = sys.target_os();
    // Known limit: no home means no roots, even with XDG_DATA_HOME set; revisit if a real system lacks HOME.
    let Some(home) = sys.home_dir() else {
        return;
    };
    let at = |base: &str, rel: &str| join(os, base, rel);
    let data = sys
        .env("XDG_DATA_HOME")
        .filter(|d| d.starts_with('/'))
        .unwrap_or_else(|| at(&home, ".local/share"));
    let flatpak = at(&home, &format!(".var/app/{FLATPAK_ID}"));
    let roots = [
        (at(&home, ".steam/steam"), Source::Steam),
        (at(&home, ".steam/root"), Source::Steam),
        (at(&data, "Steam"), Source::Steam),
        (at(&home, ".steam/debian-installation"), Source::Steam),
        (at(&home, "Steam"), Source::Steam),
        (at(&flatpak, ".local/share/Steam"), Source::FlatpakSteam),
        (at(&flatpak, "data/Steam"), Source::FlatpakSteam),
        (
            at(&home, "snap/steam/common/.local/share/Steam"),
            Source::Steam,
        ),
        (
            at(&home, ".snap/data/steam/common/.local/share/Steam"),
            Source::Steam,
        ),
    ];
    let mut seen = HashSet::new();
    for (root, source) in roots {
        let root = if sys.exists(&child(sys, &root, "steamapps")) {
            sys.canonical(&root).unwrap_or(root)
        } else {
            normalize(os, &root)
        };
        if seen.insert(root.clone()) {
            for (steamapps, has_manifest) in search_root(sys, &root, source, out) {
                search_compatdata(sys, &steamapps, has_manifest, out);
            }
        }
    }
}

/// Probes every `compatdata/<id>` in a library's `steamapps`, never by a fixed shortcut ID, and
/// lists the store prefix's would-be user dir when the library holds the app manifest (ADRs 0003, 0015).
fn search_compatdata(sys: &impl System, steamapps: &str, has_manifest: bool, out: &mut Candidates) {
    let os = sys.target_os();
    let compatdata = join(os, steamapps, "compatdata");
    for id in sys.list_dir(&compatdata) {
        let note = if id == APP_ID {
            format!("compatdata/{id} (store app)")
        } else if is_shortcut_id(&id) {
            format!("compatdata/{id} (shortcut)")
        } else {
            format!("compatdata/{id}")
        };
        probe(
            sys,
            &join(os, &compatdata, &id),
            Source::ProtonPrefix,
            &note,
            out,
        );
    }
    let store = join(
        os,
        &compatdata,
        &format!("{APP_ID}/pfx/drive_c/users/steamuser/Documents/My Games/Path of Exile 2"),
    );
    if has_manifest && !sys.exists(&store) {
        let note = format!("compatdata/{APP_ID} (store app); created on first launch");
        out.user_dir(store, Source::ProtonPrefix, Some(note));
    }
}

/// A number of at least 2^31: bit 31 marks a non-Steam shortcut (linux-paths.md Q4).
fn is_shortcut_id(id: &str) -> bool {
    !id.is_empty()
        && id.bytes().all(|b| b.is_ascii_digit())
        // Too long for a u64 is still at least 2^31.
        && id.parse::<u64>().map_or(true, |n| n >= 1 << 31)
}

/// A Steam root's installs, one per library holding the app manifest, or else its would-be
/// install (ADR 0015). Returns each existing library's `steamapps` and whether it holds the app
/// manifest.
fn search_root(
    sys: &impl System,
    root: &str,
    source: Source,
    out: &mut Candidates,
) -> Vec<(String, bool)> {
    let os = sys.target_os();
    let root_steamapps = child(sys, root, "steamapps");
    let mut libraries = vec![root.to_owned()];
    let mut errors = String::new();
    let lists = [
        join(os, &root_steamapps, "libraryfolders.vdf"),
        join(os, root, "config/libraryfolders.vdf"),
    ];
    for list in lists {
        match read(sys, &list) {
            Ok(text) => {
                for path in library_paths(&text).iter().filter_map(|p| absolute(os, p)) {
                    let remapped = sys
                        .home_dir()
                        .filter(|_| source == Source::FlatpakSteam)
                        .and_then(|home| flatpak_remap(&home, FLATPAK_ID, &path));
                    libraries.push(path);
                    libraries.extend(remapped.map(|path| normalize(os, &path)));
                }
            }
            Err(e) if e.kind() == ErrorKind::NotFound => {}
            Err(e) => errors.push_str(&format!("; {list}: {e}")),
        }
    }
    // Each distinct library once: probing lists the library folder, so repeats would multiply that cost.
    let mut listed = HashSet::new();
    libraries.retain(|library| listed.insert(key(os, library)));
    // Spellings `key` doesn't fold still repeat a folder, so the count is capped too; the root comes first.
    if libraries.len() > 1 + MAX_LIBRARIES {
        libraries.truncate(1 + MAX_LIBRARIES);
        errors.push_str(&format!(
            "; library list truncated to {MAX_LIBRARIES} entries"
        ));
    }
    let mut seen = HashSet::new();
    let mut found = false;
    let mut existing = Vec::new();
    for library in &libraries {
        let steamapps = child(sys, library, "steamapps");
        // Linux keeps only existing libraries, by canonical path (linux-paths.md Q1).
        let id = match os {
            TargetOs::Windows => Some(key(os, library)),
            TargetOs::Linux => sys.canonical(&steamapps),
        };
        if !id.is_some_and(|id| seen.insert(id)) {
            continue;
        }
        let manifest = join(os, &steamapps, &format!("appmanifest_{APP_ID}.acf"));
        existing.push((steamapps.clone(), sys.exists(&manifest)));
        let text = match read(sys, &manifest) {
            Ok(text) => text,
            Err(e) if e.kind() == ErrorKind::NotFound => continue,
            Err(e) => {
                errors.push_str(&format!("; {manifest}: {e}"));
                continue;
            }
        };
        let Some(dir) = installdir(&text).filter(|dir| stays_inside(dir)) else {
            continue;
        };
        out.install(
            join(os, &steamapps, &format!("common/{dir}")),
            source,
            Some(format!("app manifest in library {library}")),
        );
        found = true;
    }
    if !found {
        let why = if sys.exists(&root_steamapps) {
            format!("no app manifest for {APP_ID}")
        } else {
            "Steam root not found".to_owned()
        };
        out.install(
            join(os, &root_steamapps, "common/Path of Exile 2"),
            source,
            Some(why + &errors),
        );
    }
    existing
}

/// An untrusted `installdir` is a single folder name: no separator, drive, `.` or `..`.
fn stays_inside(dir: &str) -> bool {
    !(dir.is_empty() || dir.contains(['/', '\\', ':']) || dir == "." || dir == "..")
}

/// A launcher file with one leading byte order mark removed.
fn read(sys: &impl System, path: &str) -> io::Result<String> {
    let text = sys.read(path)?;
    Ok(match text.strip_prefix('\u{feff}') {
        Some(rest) => rest.to_owned(),
        None => text,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::locate::{
        Source,
        candidates::collect,
        discover_with,
        fake::{FakeSystem, acf, rows, vdf},
        system::{Hive, READ_CAP},
    };
    const ROOT: &str = r"C:\Program Files (x86)\Steam";
    fn windows_steam() -> FakeSystem {
        FakeSystem::windows()
            .reg(
                Hive::CurrentUser,
                r"Software\Valve\Steam",
                "SteamPath",
                "c:/program files (x86)/steam",
            )
            .reg(
                Hive::LocalMachine,
                r"SOFTWARE\WOW6432Node\Valve\Steam",
                "InstallPath",
                ROOT,
            )
            .file(
                &format!(r"{ROOT}\steamapps\libraryfolders.vdf"),
                &vdf(&[ROOT, r"D:\SteamLibrary"]),
            )
            .file(
                &format!(r"{ROOT}\config\libraryfolders.vdf"),
                &vdf(&[r"E:\Games\Steam"]),
            )
            .file(
                r"D:\SteamLibrary\steamapps\appmanifest_2694490.acf",
                &acf("Path of Exile 2"),
            )
            .file(
                r"D:\SteamLibrary\steamapps\common\Path of Exile 2\logs\Client.txt",
                "",
            )
            .file(
                r"E:\Games\Steam\steamapps\appmanifest_2694490.acf",
                &acf("PoE2 Copy"),
            )
            .dir(r"E:\Games\Steam\steamapps\common\PoE2 Copy")
    }
    /// A Windows system whose only Steam root is `D:\Steam`.
    fn d_steam() -> FakeSystem {
        FakeSystem::windows().reg(
            Hive::CurrentUser,
            r"Software\Valve\Steam",
            "SteamPath",
            "d:/steam",
        )
    }
    #[test]
    fn finds_installs_in_libraries_from_both_library_lists() {
        let d = collect(&windows_steam(), search_windows);
        assert_eq!(
            rows(&d.installs),
            [
                (
                    r"D:\SteamLibrary\steamapps\common\Path of Exile 2",
                    Source::Steam,
                    true
                ),
                (
                    r"E:\Games\Steam\steamapps\common\PoE2 Copy",
                    Source::Steam,
                    true
                ),
            ]
        );
        assert_eq!(
            d.installs[0].note.as_deref(),
            Some(r"app manifest in library D:\SteamLibrary")
        );
        assert_eq!(
            rows(&d.client_log)[0],
            (
                r"D:\SteamLibrary\steamapps\common\Path of Exile 2\logs\Client.txt",
                Source::Steam,
                true
            )
        );
    }
    #[test]
    fn unc_roots_and_libraries_are_never_probed() {
        // Sentinel installs on a share, which FakeSystem would resolve if they were probed (ADR 0024).
        let share = r"\\192.0.2.66\s";
        let sys = FakeSystem::windows()
            .reg(
                Hive::CurrentUser,
                r"Software\Valve\Steam",
                "SteamPath",
                &format!(r"{share}\Steam"),
            )
            .file(
                &format!(r"{share}\Steam\steamapps\appmanifest_2694490.acf"),
                &acf("Path of Exile 2"),
            )
            .file(
                &format!(r"{ROOT}\steamapps\libraryfolders.vdf"),
                &vdf(&[
                    &format!(r"{share}\lib"),
                    r"\\.\pipe\lib",
                    "//192.0.2.66/s/lib2",
                ]),
            )
            .file(
                &format!(r"{share}\lib\steamapps\appmanifest_2694490.acf"),
                &acf("Path of Exile 2"),
            )
            .file(
                &format!(r"{share}\lib2\steamapps\appmanifest_2694490.acf"),
                &acf("Path of Exile 2"),
            )
            .file(
                r"\\.\pipe\lib\steamapps\appmanifest_2694490.acf",
                &acf("Path of Exile 2"),
            );
        let d = collect(&sys, search_windows);
        assert_eq!(
            rows(&d.installs),
            [(
                r"C:\Program Files (x86)\Steam\steamapps\common\Path of Exile 2",
                Source::Fallback,
                false
            )]
        );
    }
    #[test]
    fn lists_the_default_folder_when_steam_is_missing() {
        let d = collect(&FakeSystem::windows(), search_windows);
        assert_eq!(
            rows(&d.installs),
            [(
                r"C:\Program Files (x86)\Steam\steamapps\common\Path of Exile 2",
                Source::Fallback,
                false
            )]
        );
        assert_eq!(d.installs[0].note.as_deref(), Some("Steam root not found"));
        let moved = collect(
            &FakeSystem::windows().env("ProgramFiles(x86)", r"E:\Apps (x86)"),
            search_windows,
        );
        assert_eq!(
            rows(&moved.installs)[0].0,
            r"E:\Apps (x86)\Steam\steamapps\common\Path of Exile 2"
        );
    }
    #[test]
    fn a_root_without_the_manifest_says_why() {
        let d = collect(&d_steam().dir(r"D:\Steam\steamapps"), search_windows);
        assert_eq!(
            rows(&d.installs)[0],
            (
                r"d:\steam\steamapps\common\Path of Exile 2",
                Source::Steam,
                false
            )
        );
        assert_eq!(
            d.installs[0].note.as_deref(),
            Some("no app manifest for 2694490")
        );
        let big = d_steam().file(
            r"D:\Steam\steamapps\libraryfolders.vdf",
            &"x".repeat(READ_CAP as usize + 1),
        );
        let d = collect(&big, search_windows);
        assert_eq!(
            d.installs[0].note.as_deref(),
            Some(
                r"no app manifest for 2694490; d:\steam\steamapps\libraryfolders.vdf: larger than 1 MiB"
            )
        );
    }
    #[test]
    fn an_unreadable_app_manifest_is_noted() {
        let big = d_steam().file(
            r"D:\Steam\steamapps\appmanifest_2694490.acf",
            &"x".repeat(READ_CAP as usize + 1),
        );
        let d = collect(&big, search_windows);
        assert_eq!(
            d.installs[0].note.as_deref(),
            Some(
                r"no app manifest for 2694490; d:\steam\steamapps\appmanifest_2694490.acf: larger than 1 MiB"
            )
        );
        let missing = collect(&d_steam().dir(r"D:\Steam\steamapps"), search_windows);
        assert_eq!(
            missing.installs[0].note.as_deref(),
            Some("no app manifest for 2694490")
        );
    }
    #[test]
    fn discover_with_runs_the_windows_steam_search() {
        let d = discover_with(&windows_steam());
        assert!(rows(&d.installs).contains(&(
            r"D:\SteamLibrary\steamapps\common\Path of Exile 2",
            Source::Steam,
            true
        )));
    }
    #[test]
    fn empty_library_paths_and_installdirs_count_as_absent() {
        let sys = d_steam()
            .file(r"D:\Steam\steamapps\libraryfolders.vdf", &vdf(&[""]))
            .file(r"D:\Steam\steamapps\appmanifest_2694490.acf", &acf(""))
            // Where an empty library path would lead: `\steamapps\…` on the current drive.
            .file(
                r"\steamapps\appmanifest_2694490.acf",
                &acf("Path of Exile 2"),
            );
        let d = collect(&sys, search_windows);
        assert_eq!(
            rows(&d.installs)[0],
            (
                r"d:\steam\steamapps\common\Path of Exile 2",
                Source::Steam,
                false
            )
        );
        assert_eq!(
            d.installs[0].note.as_deref(),
            Some("no app manifest for 2694490")
        );
    }
    #[test]
    fn an_installdir_that_leaves_the_library_is_skipped() {
        for dir in [
            r"..\..\Windows",
            "../x",
            "..",
            ".",
            r"C:\Elsewhere",
            "C:x",
            "/x",
        ] {
            let sys = d_steam().file(r"D:\Steam\steamapps\appmanifest_2694490.acf", &acf(dir));
            let d = collect(&sys, search_windows);
            assert_eq!(
                d.installs[0].note.as_deref(),
                Some("no app manifest for 2694490"),
                "{dir:?}"
            );
        }
        let sys = d_steam().file(r"D:\Steam\steamapps\appmanifest_2694490.acf", &acf("a..b"));
        let d = collect(&sys, search_windows);
        assert_eq!(rows(&d.installs)[0].0, r"d:\steam\steamapps\common\a..b");
    }
    #[test]
    fn relative_steam_roots_and_library_paths_are_skipped() {
        // Sentinels where a relative value would lead if it were kept (FakeSystem makes `steam` a drive).
        let sys = FakeSystem::windows()
            .reg(
                Hive::CurrentUser,
                r"Software\Valve\Steam",
                "SteamPath",
                "steam",
            )
            .file(
                r"steam\steamapps\appmanifest_2694490.acf",
                &acf("Path of Exile 2"),
            );
        let d = collect(&sys, search_windows);
        assert_eq!(
            rows(&d.installs),
            [(
                r"C:\Program Files (x86)\Steam\steamapps\common\Path of Exile 2",
                Source::Fallback,
                false
            )]
        );
        let sys = d_steam()
            .file(r"D:\Steam\steamapps\libraryfolders.vdf", &vdf(&["lib"]))
            .file(
                r"lib\steamapps\appmanifest_2694490.acf",
                &acf("Path of Exile 2"),
            );
        let d = collect(&sys, search_windows);
        assert_eq!(
            rows(&d.installs),
            [
                (
                    r"d:\steam\steamapps\common\Path of Exile 2",
                    Source::Steam,
                    false
                ),
                (
                    r"C:\Program Files (x86)\Steam\steamapps\common\Path of Exile 2",
                    Source::Fallback,
                    false
                ),
            ]
        );
    }
    #[test]
    fn a_leading_byte_order_mark_is_ignored() {
        let bom = |text: String| format!("\u{feff}{text}");
        let sys = d_steam()
            .file(
                r"D:\Steam\steamapps\libraryfolders.vdf",
                &bom(vdf(&[r"D:\SteamLibrary"])),
            )
            .file(
                r"D:\SteamLibrary\steamapps\appmanifest_2694490.acf",
                &bom(acf("Path of Exile 2")),
            );
        let d = collect(&sys, search_windows);
        assert_eq!(
            rows(&d.installs)[0].0,
            r"D:\SteamLibrary\steamapps\common\Path of Exile 2"
        );
    }

    const H: &str = "/home/<user>";
    const NATIVE: &str = "/home/<user>/.local/share/Steam";
    const FLATPAK: &str = "/home/<user>/.var/app/com.valvesoftware.Steam/.local/share/Steam";
    fn native() -> FakeSystem {
        FakeSystem::linux(H)
            .dir(&format!("{NATIVE}/steamapps"))
            .link("/home/<user>/.steam/steam", NATIVE)
            .link("/home/<user>/.steam/root", NATIVE)
    }
    #[test]
    fn native_root_extra_library_and_the_missing_roots() {
        let sys = native()
            .file(
                &format!("{NATIVE}/steamapps/libraryfolders.vdf"),
                &vdf(&[NATIVE, "/mnt/games/SteamLibrary"]),
            )
            .file(
                "/mnt/games/SteamLibrary/steamapps/appmanifest_2694490.acf",
                &acf("Path of Exile 2"),
            )
            .file(
                "/mnt/games/SteamLibrary/steamapps/common/Path of Exile 2/logs/Client.txt",
                "",
            );
        let d = collect(&sys, search_linux);
        assert_eq!(
            rows(&d.installs),
            [
                (
                    "/mnt/games/SteamLibrary/steamapps/common/Path of Exile 2",
                    Source::Steam,
                    true
                ),
                (
                    "/home/<user>/.steam/debian-installation/steamapps/common/Path of Exile 2",
                    Source::Steam,
                    false
                ),
                (
                    "/home/<user>/Steam/steamapps/common/Path of Exile 2",
                    Source::Steam,
                    false
                ),
                (
                    "/home/<user>/.var/app/com.valvesoftware.Steam/.local/share/Steam/steamapps/common/Path of Exile 2",
                    Source::FlatpakSteam,
                    false
                ),
                (
                    "/home/<user>/.var/app/com.valvesoftware.Steam/data/Steam/steamapps/common/Path of Exile 2",
                    Source::FlatpakSteam,
                    false
                ),
                (
                    "/home/<user>/snap/steam/common/.local/share/Steam/steamapps/common/Path of Exile 2",
                    Source::Steam,
                    false
                ),
                (
                    "/home/<user>/.snap/data/steam/common/.local/share/Steam/steamapps/common/Path of Exile 2",
                    Source::Steam,
                    false
                ),
            ]
        );
        assert_eq!(d.installs[1].note.as_deref(), Some("Steam root not found"));
    }
    #[test]
    fn a_repeated_library_is_listed_once() {
        // Each listing of a big library costs its entry count, so repeats must not multiply it.
        let lib = "/mnt/games";
        let list = vdf(&[lib, lib, "/mnt/games/", lib]);
        let sys = native()
            .file(&format!("{NATIVE}/steamapps/libraryfolders.vdf"), &list)
            .file(&format!("{NATIVE}/config/libraryfolders.vdf"), &list)
            .file(
                "/mnt/games/steamapps/appmanifest_2694490.acf",
                &acf("Path of Exile 2"),
            );
        let d = collect(&sys, search_linux);
        assert_eq!(sys.list_count(lib), 1);
        assert_eq!(
            rows(&d.installs)[0],
            (
                "/mnt/games/steamapps/common/Path of Exile 2",
                Source::Steam,
                false
            )
        );
    }
    #[test]
    fn a_library_list_is_capped_at_64_libraries() {
        // Spellings `key` doesn't fold (`/x//big`, `/x/./big`) would each list the folder again (ADR 0026).
        let libs: Vec<String> = (0..70).map(|i| format!("/mnt/l{i:02}")).collect();
        let refs: Vec<&str> = libs.iter().map(String::as_str).collect();
        let sys = native().file(
            &format!("{NATIVE}/steamapps/libraryfolders.vdf"),
            &vdf(&refs),
        );
        let d = collect(&sys, search_linux);
        let listed: Vec<usize> = libs.iter().map(|l| sys.list_count(l)).collect();
        assert_eq!(listed, [vec![1; 64], vec![0; 6]].concat());
        assert_eq!(
            d.installs[0].note.as_deref(),
            Some("no app manifest for 2694490; library list truncated to 64 entries")
        );
    }
    #[test]
    fn xdg_data_home_moves_the_native_root() {
        let sys = FakeSystem::linux(H)
            .env("XDG_DATA_HOME", "/data/xdg")
            .file(
                "/data/xdg/Steam/steamapps/appmanifest_2694490.acf",
                &acf("Path of Exile 2"),
            )
            .dir("/data/xdg/Steam/steamapps/common/Path of Exile 2");
        let d = collect(&sys, search_linux);
        assert_eq!(
            rows(&d.installs)[0],
            (
                "/data/xdg/Steam/steamapps/common/Path of Exile 2",
                Source::Steam,
                true
            )
        );
        assert!(rows(&d.installs).iter().all(|r| !r.0.starts_with(NATIVE)));
    }
    #[test]
    fn relative_library_paths_are_skipped() {
        // A sentinel install at `/rel/lib`, where FakeSystem resolves `rel/lib`.
        let sys = native()
            .file(
                &format!("{NATIVE}/steamapps/libraryfolders.vdf"),
                &vdf(&["rel/lib"]),
            )
            .file(
                "/rel/lib/steamapps/appmanifest_2694490.acf",
                &acf("Path of Exile 2"),
            )
            .dir("/rel/lib/steamapps/common/Path of Exile 2");
        let d = collect(&sys, search_linux);
        assert!(
            rows(&d.installs)
                .iter()
                .all(|r| !r.2 && r.0.starts_with('/')),
            "{:?}",
            rows(&d.installs)
        );
    }
    #[test]
    fn a_relative_xdg_data_home_falls_back_to_the_default() {
        // A sentinel install at `/data`, where FakeSystem resolves `data`.
        let sys = FakeSystem::linux(H)
            .env("XDG_DATA_HOME", "data")
            .file(
                "/data/Steam/steamapps/appmanifest_2694490.acf",
                &acf("Path of Exile 2"),
            )
            .dir("/data/Steam/steamapps/common/Path of Exile 2")
            .file(
                &format!("{NATIVE}/steamapps/appmanifest_2694490.acf"),
                &acf("Path of Exile 2"),
            )
            .dir(&format!("{NATIVE}/steamapps/common/Path of Exile 2"));
        let d = collect(&sys, search_linux);
        let found: Vec<_> = rows(&d.installs).into_iter().filter(|r| r.2).collect();
        assert_eq!(
            found,
            [(
                "/home/<user>/.local/share/Steam/steamapps/common/Path of Exile 2",
                Source::Steam,
                true
            )]
        );
    }
    #[test]
    fn flatpak_library_paths_are_remapped_and_sandbox_links_ignored() {
        let sys = FakeSystem::linux(H)
            .file(&format!("{FLATPAK}/steamapps/libraryfolders.vdf"), &vdf(&[NATIVE, "/var/data/SteamLibrary", "/run/media/<user>/disk/SteamLibrary"]))
            .file("/home/<user>/.var/app/com.valvesoftware.Steam/data/SteamLibrary/steamapps/appmanifest_2694490.acf", &acf("Path of Exile 2"))
            .dir("/home/<user>/.var/app/com.valvesoftware.Steam/data/SteamLibrary/steamapps/common/Path of Exile 2")
            .file("/run/media/<user>/disk/SteamLibrary/steamapps/appmanifest_2694490.acf", &acf("Path of Exile 2"))
            .dir("/run/media/<user>/disk/SteamLibrary/steamapps/common/Path of Exile 2")
            .link("/home/<user>/.var/app/com.valvesoftware.Steam/.steam/root", NATIVE);
        let d = collect(&sys, search_linux);
        let found: Vec<_> = rows(&d.installs).into_iter().filter(|r| r.2).collect();
        assert_eq!(
            found,
            [
                (
                    "/home/<user>/.var/app/com.valvesoftware.Steam/data/SteamLibrary/steamapps/common/Path of Exile 2",
                    Source::FlatpakSteam,
                    true
                ),
                (
                    "/run/media/<user>/disk/SteamLibrary/steamapps/common/Path of Exile 2",
                    Source::FlatpakSteam,
                    true
                ),
            ]
        );
        assert!(
            rows(&d.installs)
                .iter()
                .all(|r| !r.0.contains("com.valvesoftware.Steam/.steam/"))
        );
    }
    #[test]
    fn snap_root_config_library_list_and_old_steamapps_case() {
        let snap = "/home/<user>/snap/steam/common/.local/share/Steam";
        let sys = FakeSystem::linux(H)
            .dir(&format!("{snap}/steamapps"))
            .file(
                &format!("{snap}/config/libraryfolders.vdf"),
                &vdf(&["/mnt/old"]),
            )
            .file(
                "/mnt/old/SteamApps/appmanifest_2694490.acf",
                &acf("Path of Exile 2"),
            )
            .dir("/mnt/old/SteamApps/common/Path of Exile 2");
        let d = collect(&sys, search_linux);
        assert_eq!(
            rows(&d.installs)[0],
            (
                "/mnt/old/SteamApps/common/Path of Exile 2",
                Source::Steam,
                true
            )
        );
    }
    #[test]
    fn discover_with_runs_the_linux_steam_search() {
        let d = discover_with(&native());
        assert_eq!(
            rows(&d.installs)[0].0,
            "/home/<user>/.local/share/Steam/steamapps/common/Path of Exile 2"
        );
        assert_eq!(
            d.installs[0].note.as_deref(),
            Some("no app manifest for 2694490")
        );
    }
}
