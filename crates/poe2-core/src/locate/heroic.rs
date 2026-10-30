//! Heroic's recorded and default Wine prefixes (docs/research/linux-paths.md Q5, Unknown, Probe recipe 3); ADRs 0003, 0004, 0005, 0015, 0018.

use super::Source;
use super::candidates::Candidates;
use super::json::string_value;
use super::prefix::{probe, probe_default};
use super::system::System;
use super::tpath::{expand_tilde, flatpak_remap, join};
use std::io::ErrorKind;

/// Heroic Flatpak's app ID.
const FLATPAK_ID: &str = "com.heroicgameslauncher.hgl";

/// Every `winePrefix` recorded in the native and Flatpak config dirs, unfiltered by game (ADR 0003),
/// then the default prefix folders, then the fixed prefix, noted with any read errors (ADR 0015).
pub(crate) fn search(sys: &impl System, out: &mut Candidates) {
    let os = sys.target_os();
    let Some(home) = sys.home_dir() else {
        return;
    };
    let config = sys
        .env("XDG_CONFIG_HOME")
        .filter(|dir| dir.starts_with('/'))
        .unwrap_or_else(|| join(os, &home, ".config"));
    let dirs = [
        (join(os, &config, "heroic"), false),
        (
            join(os, &home, &format!(".var/app/{FLATPAK_ID}/config/heroic")),
            true,
        ),
    ];
    let mut errors = String::new();
    for (dir, flatpak) in dirs {
        let games = join(os, &dir, "GamesConfig");
        let files = sys
            .list_dir(&games)
            .into_iter()
            .filter(|name| name.ends_with(".json"))
            .map(|name| {
                (
                    join(os, &games, &name),
                    format!("Heroic GamesConfig/{name}"),
                )
            })
            .chain([(
                join(os, &dir, "config.json"),
                "Heroic config.json".to_owned(),
            )]);
        for (file, note) in files {
            let text = match sys.read(&file) {
                Ok(text) => text,
                Err(e) if e.kind() == ErrorKind::NotFound => continue,
                Err(e) => {
                    errors.push_str(&format!("; {file}: {e}"));
                    continue;
                }
            };
            // Untrusted: an empty or relative prefix is skipped, never resolved against the cwd.
            let Some(prefix) = string_value(&text, "winePrefix")
                .map(|p| expand_tilde(&home, &p))
                .filter(|p| p.starts_with('/'))
            else {
                continue;
            };
            let remapped = flatpak
                .then(|| flatpak_remap(&home, FLATPAK_ID, &prefix))
                .flatten();
            for prefix in [Some(prefix), remapped].into_iter().flatten() {
                probe(sys, &prefix, Source::Heroic, &note, out);
            }
        }
    }
    let prefixes = join(os, &home, "Games/Heroic/Prefixes");
    let default = join(os, &prefixes, "Path of Exile 2");
    for base in [prefixes.clone(), join(os, &prefixes, "default")] {
        for name in sys.list_dir(&base) {
            let prefix = join(os, &base, &name);
            // The default prefix is probed once, below, so its row keeps the read errors.
            if prefix != default {
                probe(sys, &prefix, Source::Heroic, "Heroic Prefixes folder", out);
            }
        }
    }
    probe_default(
        sys,
        &default,
        Source::Heroic,
        &format!("Heroic default prefix{errors}"),
        out,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::locate::{
        Source,
        candidates::collect,
        discover_with,
        fake::{FakeSystem, rows},
        system::READ_CAP,
    };
    const H: &str = "/home/<user>";
    const CFG: &str = "/home/<user>/.config/heroic";
    const PFX: &str = "/home/<user>/Games/Heroic/Prefixes";
    const UD: &str = "drive_c/users/steamuser/Documents/My Games/Path of Exile 2";
    const GGG: &str = "drive_c/Program Files (x86)/Grinding Gear Games/Path of Exile 2";
    #[test]
    fn recorded_prefixes_with_and_without_pfx() {
        let sys = FakeSystem::linux(H)
            .file(&format!("{CFG}/GamesConfig/abc123.json"), r#"{ "abc123": { "winePrefix": "~/Games/Heroic/Prefixes/PoE2", "wineVersion": { "type": "proton" } }, "version": "v0", "explicit": true }"#)
            .dir(&format!("{PFX}/PoE2/pfx/{UD}"))
            .file(&format!("{CFG}/config.json"), r#"{ "defaultSettings": { "winePrefix": "/home/<user>/Games/Heroic/Prefixes/shared" }, "version": "v0" }"#)
            .dir(&format!("{PFX}/shared/drive_c/Program Files/Grinding Gear Games/Path of Exile 2"));
        let d = collect(&sys, search);
        let ud = format!("{PFX}/PoE2/pfx/{UD}");
        assert_eq!(rows(&d.user_dir), [(ud.as_str(), Source::Heroic, true)]);
        assert_eq!(
            d.user_dir[0].note.as_deref(),
            Some("Heroic GamesConfig/abc123.json")
        );
        let shared =
            format!("{PFX}/shared/drive_c/Program Files/Grinding Gear Games/Path of Exile 2");
        assert_eq!(
            rows(&d.installs)[0],
            (shared.as_str(), Source::Heroic, true)
        );
        assert_eq!(d.installs[0].note.as_deref(), Some("Heroic config.json"));
    }
    #[test]
    fn flatpak_heroic_prefixes_are_remapped() {
        let sys = FakeSystem::linux(H)
            .file("/home/<user>/.var/app/com.heroicgameslauncher.hgl/config/heroic/GamesConfig/x.json", r#"{"x": {"winePrefix": "/home/<user>/WinePrefixes/poe2"}}"#)
            .dir(&format!("/home/<user>/.var/app/com.heroicgameslauncher.hgl/WinePrefixes/poe2/{UD}"));
        let ud =
            format!("/home/<user>/.var/app/com.heroicgameslauncher.hgl/WinePrefixes/poe2/{UD}");
        assert_eq!(
            rows(&collect(&sys, search).user_dir),
            [(ud.as_str(), Source::Heroic, true)]
        );
    }
    #[test]
    fn default_prefix_folders_are_searched() {
        let sys = FakeSystem::linux(H).dir(&format!(
            "{PFX}/default/Path of Exile 2/drive_c/users/<user>/Documents/My Games/Path of Exile 2"
        ));
        let d = collect(&sys, search);
        assert_eq!(d.user_dir.len(), 1);
        assert_eq!(
            d.user_dir[0].note.as_deref(),
            Some("Heroic Prefixes folder")
        );
    }
    #[test]
    fn nothing_found_lists_the_default_prefix_and_bad_json_is_skipped() {
        let d = collect(
            &FakeSystem::linux(H).file(&format!("{CFG}/GamesConfig/bad.json"), "{"),
            search,
        );
        let row = format!("{PFX}/Path of Exile 2/{GGG}");
        assert_eq!(rows(&d.installs), [(row.as_str(), Source::Heroic, false)]);
        assert_eq!(d.installs[0].note.as_deref(), Some("Heroic default prefix"));
        assert!(d.user_dir.is_empty());
    }
    #[test]
    fn xdg_config_home_is_honoured() {
        let sys = FakeSystem::linux(H)
            .env("XDG_CONFIG_HOME", "/cfg")
            .file(
                "/cfg/heroic/GamesConfig/y.json",
                r#"{"y": {"winePrefix": "/p"}}"#,
            )
            .dir("/p/drive_c/users/<user>/Documents/My Games/Path of Exile 2");
        assert_eq!(collect(&sys, search).user_dir.len(), 1);
    }
    #[test]
    fn a_relative_xdg_config_home_falls_back_to_dot_config() {
        let sys = FakeSystem::linux(H)
            .env("XDG_CONFIG_HOME", "cfg")
            .file(
                &format!("{CFG}/GamesConfig/y.json"),
                r#"{"y": {"winePrefix": "/p"}}"#,
            )
            .dir(&format!("/p/{UD}"));
        assert_eq!(collect(&sys, search).user_dir.len(), 1);
    }
    #[test]
    fn native_prefixes_are_not_remapped() {
        let sys = FakeSystem::linux(H)
            .file(
                &format!("{CFG}/GamesConfig/x.json"),
                r#"{"x": {"winePrefix": "/home/<user>/WinePrefixes/poe2"}}"#,
            )
            .dir(&format!(
                "/home/<user>/.var/app/com.heroicgameslauncher.hgl/WinePrefixes/poe2/{UD}"
            ));
        assert!(collect(&sys, search).user_dir.is_empty());
    }
    #[test]
    fn empty_and_relative_prefixes_are_skipped() {
        let sys = FakeSystem::linux(H)
            .file(
                &format!("{CFG}/GamesConfig/a.json"),
                r#"{"a": {"winePrefix": ""}}"#,
            )
            .file(
                &format!("{CFG}/GamesConfig/b.json"),
                r#"{"b": {"winePrefix": "rel/poe2"}}"#,
            )
            .file(
                &format!("{CFG}/GamesConfig/notes.txt"),
                r#"{"winePrefix": "/p"}"#,
            )
            .dir(&format!("/p/{UD}"));
        assert!(collect(&sys, search).user_dir.is_empty());
    }
    #[test]
    fn bom_and_unreadable_files() {
        let sys = FakeSystem::linux(H)
            .file(
                &format!("{CFG}/GamesConfig/a.json"),
                "\u{feff}{\"a\": {\"winePrefix\": \"/p\"}}",
            )
            .dir(&format!("/p/{UD}"))
            .file(
                &format!("{CFG}/config.json"),
                &"x".repeat(READ_CAP as usize + 1),
            );
        let d = collect(&sys, search);
        assert_eq!(d.user_dir.len(), 1);
        assert_eq!(
            d.installs[0].note.as_deref(),
            Some(
                "Heroic default prefix; /home/<user>/.config/heroic/config.json: larger than 1 MiB"
            )
        );
    }
    #[test]
    fn an_existing_default_prefix_is_probed_once_and_keeps_read_errors() {
        let def = format!("{PFX}/Path of Exile 2");
        let sys = FakeSystem::linux(H)
            .file(
                &format!("{CFG}/config.json"),
                &"x".repeat(READ_CAP as usize + 1),
            )
            .dir(&format!("{def}/{GGG}"));
        let d = collect(&sys, search);
        assert_eq!(sys.list_count(&format!("{def}/drive_c/users")), 1);
        let row = format!("{def}/{GGG}");
        assert_eq!(rows(&d.installs), [(row.as_str(), Source::Heroic, true)]);
        assert_eq!(
            d.installs[0].note.as_deref(),
            Some(
                "Heroic default prefix; /home/<user>/.config/heroic/config.json: larger than 1 MiB"
            )
        );
    }
    #[test]
    fn discover_with_runs_heroic_on_linux() {
        let d = discover_with(&FakeSystem::linux(H));
        let row = format!("{PFX}/Path of Exile 2/{GGG}");
        assert!(rows(&d.installs).iter().any(|r| r.0 == row));
    }
}
