//! Standalone installs, Epic manifests and the Documents user dirs on Windows (docs/research/windows-paths.md Q3–Q6, Conflicting 2 and 4); ADRs 0006, 0015, 0018, 0020.

use super::Source;
use super::candidates::Candidates;
use super::json::{bool_value, string_value};
use super::system::{Hive, System};
use super::tpath::{absolute, join};

/// PoE2's Epic catalog namespace, from Epic's own API (windows-paths.md Q4).
const EPIC_NAMESPACE: &str = "58a18be6bbba41a0bf52b014bb81d33b";

/// The game's folder under a Documents folder (windows-paths.md Q6).
const USER_DIR: &str = r"My Games\Path of Exile 2";

/// The standalone registry value and default folders, then every matching Epic manifest, then the
/// user dirs under the Documents known folder and `%USERPROFILE%\Documents`.
pub(crate) fn search(sys: &impl System, out: &mut Candidates) {
    let os = sys.target_os();
    let env_or =
        |name: &str, default: &str| sys.env_path(name).unwrap_or_else(|| default.to_owned());
    let registry = sys.registry_string(
        Hive::CurrentUser,
        r"Software\GrindingGearGames\Path of Exile 2",
        "InstallLocation",
    );
    if let Some(dir) = registry.and_then(|dir| absolute(os, &dir)) {
        let note = Some("registry InstallLocation".to_owned());
        out.install(dir, Source::Standalone, note);
    }
    for (name, default) in [
        ("ProgramFiles(x86)", r"C:\Program Files (x86)"),
        ("ProgramFiles", r"C:\Program Files"),
    ] {
        let dir = join(
            os,
            &env_or(name, default),
            r"Grinding Gear Games\Path of Exile 2",
        );
        out.install(
            dir,
            Source::Fallback,
            Some("default standalone folder".to_owned()),
        );
    }

    let manifests = join(
        os,
        &env_or("PROGRAMDATA", r"C:\ProgramData"),
        r"Epic\EpicGamesLauncher\Data\Manifests",
    );
    let items = sys.list_dir(&manifests).into_iter().filter(|name| {
        name.rsplit_once('.')
            .is_some_and(|(_, ext)| ext.eq_ignore_ascii_case("item"))
    });
    for name in items {
        // Unreadable manifests are skipped: Epic has no fixed row to note them on (ADR 0020).
        let Ok(text) = sys.read(&join(os, &manifests, &name)) else {
            continue;
        };
        let poe2 = string_value(&text, "CatalogNamespace").as_deref() == Some(EPIC_NAMESPACE)
            || string_value(&text, "DisplayName").is_some_and(|n| n.starts_with("Path of Exile 2"));
        let dir = string_value(&text, "InstallLocation").and_then(|dir| absolute(os, &dir));
        if let Some(dir) = dir.filter(|_| poe2) {
            let incomplete = bool_value(&text, "bIsIncompleteInstall") == Some(true);
            let suffix = if incomplete {
                "; incomplete install"
            } else {
                ""
            };
            out.install(
                dir,
                Source::Epic,
                Some(format!("Epic manifest {name}{suffix}")),
            );
        }
    }

    if let Some(documents) = sys.documents_folder() {
        let note = Some("Documents known folder".to_owned());
        out.user_dir(join(os, &documents, USER_DIR), Source::KnownFolder, note);
    }
    if let Some(profile) = sys.env_path("USERPROFILE") {
        let note = Some(r"%USERPROFILE%\Documents".to_owned());
        let dir = join(os, &profile, &format!(r"Documents\{USER_DIR}"));
        out.user_dir(dir, Source::Fallback, note);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::locate::{
        Source,
        candidates::collect,
        discover_with,
        fake::{FakeSystem, rows},
        system::Hive,
    };
    const M: &str = r"C:\ProgramData\Epic\EpicGamesLauncher\Data\Manifests";
    #[test]
    fn standalone_registry_value_and_default_folders() {
        let sys = FakeSystem::windows()
            .reg(
                Hive::CurrentUser,
                r"Software\GrindingGearGames\Path of Exile 2",
                "InstallLocation",
                r"D:\Games\Path of Exile 2\",
            )
            .dir(r"D:\Games\Path of Exile 2");
        let d = collect(&sys, search);
        assert_eq!(
            rows(&d.installs),
            [
                (r"D:\Games\Path of Exile 2", Source::Standalone, true),
                (
                    r"C:\Program Files (x86)\Grinding Gear Games\Path of Exile 2",
                    Source::Fallback,
                    false
                ),
                (
                    r"C:\Program Files\Grinding Gear Games\Path of Exile 2",
                    Source::Fallback,
                    false
                ),
            ]
        );
        assert_eq!(
            d.installs[0].note.as_deref(),
            Some("registry InstallLocation")
        );
        assert_eq!(
            d.installs[1].note.as_deref(),
            Some("default standalone folder")
        );
        let moved = collect(
            &FakeSystem::windows()
                .env("ProgramFiles(x86)", r"E:\PF86")
                .env("ProgramFiles", r"E:\PF"),
            search,
        );
        assert_eq!(
            rows(&moved.installs)[..2],
            [
                (
                    r"E:\PF86\Grinding Gear Games\Path of Exile 2",
                    Source::Fallback,
                    false
                ),
                (
                    r"E:\PF\Grinding Gear Games\Path of Exile 2",
                    Source::Fallback,
                    false
                )
            ]
        );
    }
    #[test]
    fn epic_manifests_match_namespace_or_display_name() {
        let sys = FakeSystem::windows()
            .file(&format!(r"{M}\AAAA.item"), r#"{"DisplayName": "Other Game", "CatalogNamespace": "ffff", "InstallLocation": "D:\\Epic Games\\Other"}"#)
            .file(&format!(r"{M}\BBBB.item"), r#"{"DisplayName": "Path of Exile 2", "CatalogNamespace": "58a18be6bbba41a0bf52b014bb81d33b", "InstallLocation": "D:\\Epic Games\\PathOfExile2", "bIsIncompleteInstall": false}"#)
            .file(&format!(r"{M}\CCCC.ITEM"), r#"{"DisplayName": "Path of Exile 2 Free Weekend", "CatalogNamespace": "0000", "InstallLocation": "E:\\POE2FreeWeekend", "bIsIncompleteInstall": true}"#)
            .file(&format!(r"{M}\notes.txt"), "{}")
            .dir(r"D:\Epic Games\PathOfExile2");
        let d = collect(&sys, search);
        let epic: Vec<_> = rows(&d.installs)
            .into_iter()
            .filter(|r| r.1 == Source::Epic)
            .collect();
        assert_eq!(
            epic,
            [
                (r"D:\Epic Games\PathOfExile2", Source::Epic, true),
                (r"E:\POE2FreeWeekend", Source::Epic, false)
            ]
        );
        let notes: Vec<_> = d
            .installs
            .iter()
            .filter(|f| f.source == Source::Epic)
            .map(|f| f.note.as_deref())
            .collect();
        assert_eq!(
            notes,
            [
                Some("Epic manifest BBBB.item"),
                Some("Epic manifest CCCC.ITEM; incomplete install")
            ]
        );
    }
    #[test]
    fn epic_untrusted_values_bom_and_programdata() {
        let ns = r#""CatalogNamespace": "58a18be6bbba41a0bf52b014bb81d33b""#;
        let item = |loc: &str| format!(r#"{{{ns}, "InstallLocation": "{loc}"}}"#);
        let m = r"F:\PD\Epic\EpicGamesLauncher\Data\Manifests";
        let sys = FakeSystem::windows()
            .env("PROGRAMDATA", r"F:\PD")
            .file(&format!(r"{m}\a.item"), &item(""))
            .file(&format!(r"{m}\b.item"), &item(r"Epic\\PoE2"))
            .file(&format!(r"{m}\c.item"), &item(r"\\Epic\\PoE2"))
            .file(&format!(r"{m}\d.item"), &item("D:"))
            .file(
                &format!(r"{m}\e.item"),
                &format!("\u{feff}{}", item("D:/Epic/PoE2/")),
            )
            .file(&format!(r"{m}\f.item"), &item(r"\\\\192.0.2.66\\s\\PoE2"))
            .file(&format!(r"{m}\g.item"), &item(r"\\\\.\\pipe\\PoE2"))
            .file(&format!(r"{m}\h.item"), &item(r"\\\\?\\D:\\PoE2"))
            .file(&format!(r"{m}\i.item"), &item("//192.0.2.66/s/PoE2"));
        let d = collect(&sys, search);
        let epic: Vec<_> = rows(&d.installs)
            .into_iter()
            .filter(|r| r.1 == Source::Epic)
            .map(|r| r.0)
            .collect();
        // UNC, device and pipe paths are never probed (ADR 0024).
        assert_eq!(epic, [r"D:\Epic\PoE2"]);
    }
    #[test]
    fn empty_relative_and_unc_registry_values_are_skipped() {
        for value in [
            "",
            r"Games\PoE2",
            "C:",
            r"\\192.0.2.66\s\PoE2",
            r"\\.\pipe\x",
        ] {
            let sys = FakeSystem::windows().reg(
                Hive::CurrentUser,
                r"Software\GrindingGearGames\Path of Exile 2",
                "InstallLocation",
                value,
            );
            let d = collect(&sys, search);
            assert!(
                d.installs.iter().all(|f| f.source != Source::Standalone),
                "{value:?}"
            );
        }
    }
    #[test]
    fn documents_known_folder_and_the_userprofile_fallback() {
        let sys = FakeSystem::windows()
            .documents(r"C:\Users\<user>\OneDrive\Documents")
            .env("USERPROFILE", r"C:\Users\<user>")
            .dir(r"C:\Users\<user>\Documents\My Games\Path of Exile 2\BuildPlanner");
        let d = collect(&sys, search);
        assert_eq!(
            rows(&d.user_dir),
            [
                (
                    r"C:\Users\<user>\Documents\My Games\Path of Exile 2",
                    Source::Fallback,
                    true
                ),
                (
                    r"C:\Users\<user>\OneDrive\Documents\My Games\Path of Exile 2",
                    Source::KnownFolder,
                    false
                ),
            ]
        );
        assert_eq!(
            d.user_dir[0].note.as_deref(),
            Some(r"%USERPROFILE%\Documents")
        );
        assert_eq!(
            d.user_dir[1].note.as_deref(),
            Some("Documents known folder")
        );
        assert_eq!(
            rows(&d.build_planner)[0],
            (
                r"C:\Users\<user>\Documents\My Games\Path of Exile 2\BuildPlanner",
                Source::Fallback,
                true
            )
        );
        let same = FakeSystem::windows()
            .documents(r"C:\Users\<user>\Documents")
            .env("USERPROFILE", r"c:\users\<user>");
        assert_eq!(
            rows(&collect(&same, search).user_dir),
            [(
                r"C:\Users\<user>\Documents\My Games\Path of Exile 2",
                Source::KnownFolder,
                false
            )]
        );
        assert!(collect(&FakeSystem::windows(), search).user_dir.is_empty());
    }
    #[test]
    fn relative_windows_env_values_fall_back_to_the_defaults() {
        // Sentinels where FakeSystem resolves each relative value (ADR 0025).
        let sys = FakeSystem::windows()
            .env("ProgramFiles(x86)", "PF86")
            .env("ProgramFiles", "PF")
            .env("PROGRAMDATA", "PD")
            .env("USERPROFILE", r"Users\<user>")
            .dir(r"PF86\Steam\steamapps")
            .dir(r"PF86\Grinding Gear Games\Path of Exile 2")
            .dir(r"PF\Grinding Gear Games\Path of Exile 2")
            .file(
                r"PD\Epic\EpicGamesLauncher\Data\Manifests\a.item",
                r#"{"CatalogNamespace": "58a18be6bbba41a0bf52b014bb81d33b", "InstallLocation": "D:\\Epic\\PoE2"}"#,
            )
            .dir(r"Users\<user>\Documents\My Games\Path of Exile 2");
        let d = discover_with(&sys);
        let values: Vec<_> = rows(&d.installs)
            .into_iter()
            .chain(rows(&d.user_dir))
            .map(|r| r.0)
            .collect();
        assert_eq!(
            values,
            [
                r"C:\Program Files (x86)\Steam\steamapps\common\Path of Exile 2",
                r"C:\Program Files (x86)\Grinding Gear Games\Path of Exile 2",
                r"C:\Program Files\Grinding Gear Games\Path of Exile 2",
            ]
        );
    }
    #[test]
    fn discover_with_runs_this_search_after_steam_on_windows() {
        let d = discover_with(&FakeSystem::windows());
        let sources: Vec<_> = d.installs.iter().map(|f| f.source).collect();
        // The Steam default folder's would-be install, then both standalone default folders.
        assert_eq!(sources, [Source::Fallback; 3]);
        assert_eq!(
            rows(&d.installs)[1].0,
            r"C:\Program Files (x86)\Grinding Gear Games\Path of Exile 2"
        );
    }
}
