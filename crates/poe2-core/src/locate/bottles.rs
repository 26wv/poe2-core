//! Bottles' bottles and custom bottles folder (docs/research/linux-paths.md Q7, Unknown); ADRs 0003, 0004, 0005, 0015, 0018, 0019.

use super::Source;
use super::candidates::Candidates;
use super::prefix::probe;
use super::system::System;
use super::tpath::{flatpak_remap, join};
use super::yaml::scalar;
use std::io::ErrorKind;

/// Bottles Flatpak's app ID.
const FLATPAK_ID: &str = "com.usebottles.bottles";

/// Every bottle in the native and Flatpak bases, then every entry of each base's custom bottles
/// folder, unfiltered by game (ADR 0003). Bottles has no fixed location (ADR 0015); read errors go
/// on the notes of the candidates they affect (ADR 0019). `bottle.yml` is never read.
pub(crate) fn search(sys: &impl System, out: &mut Candidates) {
    let os = sys.target_os();
    let Some(home) = sys.home_dir() else {
        return;
    };
    let data = sys
        .env("XDG_DATA_HOME")
        .filter(|dir| dir.starts_with('/'))
        .unwrap_or_else(|| join(os, &home, ".local/share"));
    let bases = [
        (join(os, &data, "bottles"), false),
        (
            join(os, &home, &format!(".var/app/{FLATPAK_ID}/data/bottles")),
            true,
        ),
    ];
    for (base, flatpak) in bases {
        // Untrusted: an empty or relative path is skipped, never resolved against the cwd. A
        // Flatpak path is also tried remapped into the app's folder (ADR 0004).
        let paths = |path: Option<String>| {
            let path = path.filter(|p| p.starts_with('/'));
            let remapped = path
                .as_deref()
                .filter(|_| flatpak)
                .and_then(|p| flatpak_remap(&home, FLATPAK_ID, p));
            [path, remapped].into_iter().flatten()
        };
        let mut errors = String::new();
        let custom = read(sys, &join(os, &base, "data.yml"), &mut errors)
            .and_then(|text| scalar(&text, &["custom_bottles_path"]));
        let bottles = join(os, &base, "bottles");
        for name in sys.list_dir(&bottles) {
            let dir = join(os, &bottles, &name);
            let mut note = format!("bottle {name}{errors}");
            let recorded = read(sys, &join(os, &dir, "placeholder.yml"), &mut note)
                .and_then(|text| scalar(&text, &["Path"]));
            let prefixes: Vec<String> = paths(recorded).collect();
            if prefixes.is_empty() {
                probe(sys, &dir, Source::Bottles, &note, out);
            }
            for prefix in prefixes {
                probe(sys, &prefix, Source::Bottles, &note, out);
            }
        }
        for folder in paths(custom) {
            for name in sys.list_dir(&folder) {
                let note = format!("bottle {name}{errors}");
                probe(sys, &join(os, &folder, &name), Source::Bottles, &note, out);
            }
        }
    }
}

/// `file`'s text; `None` if missing or unreadable, noting a read error in `errors` (ADR 0015).
fn read(sys: &impl System, file: &str, errors: &mut String) -> Option<String> {
    match sys.read(file) {
        Ok(text) => Some(text),
        Err(e) if e.kind() == ErrorKind::NotFound => None,
        Err(e) => {
            errors.push_str(&format!("; {file}: {e}"));
            None
        }
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
        system::READ_CAP,
    };
    const H: &str = "/home/<user>";
    const FB: &str = "/home/<user>/.var/app/com.usebottles.bottles/data/bottles";
    const B: &str = "/home/<user>/.local/share/bottles";
    const UD: &str = "drive_c/users/<user>/Documents/My Games/Path of Exile 2";
    const GGG: &str = "drive_c/Program Files (x86)/Grinding Gear Games/Path of Exile 2";
    #[test]
    fn bottles_placeholders_and_the_custom_folder() {
        let sys = FakeSystem::linux(H)
            .dir(&format!("{B}/bottles/PoE2/{UD}"))
            .file(
                &format!("{B}/bottles/Elsewhere/placeholder.yml"),
                include_str!("fixtures/bottles_placeholder.yml"),
            )
            .dir(&format!("/mnt/games/bottles/PoE2/{GGG}"))
            .file(
                &format!("{B}/data.yml"),
                include_str!("fixtures/bottles_data.yml"),
            )
            .dir(&format!("/mnt/games/bottles/Other/{UD}"));
        let d = collect(&sys, search);
        let (native, other, inst) = (
            format!("{B}/bottles/PoE2/{UD}"),
            format!("/mnt/games/bottles/Other/{UD}"),
            format!("/mnt/games/bottles/PoE2/{GGG}"),
        );
        assert_eq!(
            rows(&d.user_dir),
            [
                (native.as_str(), Source::Bottles, true),
                (other.as_str(), Source::Bottles, true)
            ]
        );
        assert_eq!(rows(&d.installs), [(inst.as_str(), Source::Bottles, true)]);
        assert_eq!(d.installs[0].note.as_deref(), Some("bottle Elsewhere"));
        assert_eq!(d.user_dir[1].note.as_deref(), Some("bottle Other"));
    }
    #[test]
    fn flatpak_bottles_remap_var_data() {
        let sys = FakeSystem::linux(H)
            .file(
                "/home/<user>/.var/app/com.usebottles.bottles/data/bottles/bottles/PoE2/placeholder.yml",
                "Path: /var/data/custom/PoE2\n",
            )
            .dir(&format!(
                "/home/<user>/.var/app/com.usebottles.bottles/data/custom/PoE2/{UD}"
            ));
        let ud = format!("/home/<user>/.var/app/com.usebottles.bottles/data/custom/PoE2/{UD}");
        assert_eq!(
            rows(&collect(&sys, search).user_dir),
            [(ud.as_str(), Source::Bottles, true)]
        );
    }
    #[test]
    fn a_steamuser_alias_is_one_user_dir_and_no_bottles_is_nothing() {
        let users = format!("{B}/bottles/PoE2/drive_c/users");
        let sys = FakeSystem::linux(H)
            .dir(&format!(
                "{users}/<user>/Documents/My Games/Path of Exile 2"
            ))
            .link(&format!("{users}/steamuser"), &format!("{users}/<user>"));
        assert_eq!(collect(&sys, search).user_dir.len(), 1);
        let empty = collect(&FakeSystem::linux(H), search);
        assert!(empty.installs.is_empty() && empty.user_dir.is_empty());
    }
    #[test]
    fn empty_and_relative_paths_are_skipped() {
        // FakeSystem resolves "" to `/` and `rel/…` to `/rel/…`, so each would match if kept.
        let sys = FakeSystem::linux(H)
            .file(&format!("{B}/bottles/A/placeholder.yml"), "Path: ''\n")
            .dir(&format!("/{UD}"))
            .dir(&format!("{B}/bottles/A/{UD}"))
            .file(
                &format!("{B}/bottles/B/placeholder.yml"),
                "Path: rel/poe2\n",
            )
            .dir(&format!("/rel/poe2/{UD}"))
            .file(&format!("{B}/data.yml"), "custom_bottles_path: custom\n")
            .dir(&format!("/custom/C/{UD}"))
            .file(&format!("{FB}/data.yml"), "custom_bottles_path: ''\n")
            .dir(&format!("/Z/{UD}"));
        let only = format!("{B}/bottles/A/{UD}");
        assert_eq!(
            rows(&collect(&sys, search).user_dir),
            [(only.as_str(), Source::Bottles, true)]
        );
    }
    #[test]
    fn xdg_data_home_is_honoured_unless_relative() {
        let sys = FakeSystem::linux(H)
            .env("XDG_DATA_HOME", "/xdg")
            .dir(&format!("/xdg/bottles/bottles/X/{UD}"));
        let x = format!("/xdg/bottles/bottles/X/{UD}");
        assert_eq!(
            rows(&collect(&sys, search).user_dir),
            [(x.as_str(), Source::Bottles, true)]
        );
        let sys = FakeSystem::linux(H)
            .env("XDG_DATA_HOME", "data")
            .dir(&format!("/data/bottles/bottles/X/{UD}"))
            .dir(&format!("{B}/bottles/Y/{UD}"));
        let y = format!("{B}/bottles/Y/{UD}");
        assert_eq!(
            rows(&collect(&sys, search).user_dir),
            [(y.as_str(), Source::Bottles, true)]
        );
    }
    #[test]
    fn bom_and_unreadable_files() {
        let big = "x".repeat(READ_CAP as usize + 1);
        let sys = FakeSystem::linux(H)
            .file(
                &format!("{FB}/bottles/A/placeholder.yml"),
                "\u{feff}Path: /p\n",
            )
            .dir(&format!("/p/{UD}"))
            .file(&format!("{B}/data.yml"), &big)
            .file(&format!("{B}/bottles/C/placeholder.yml"), &big)
            .dir(&format!("{B}/bottles/C/{UD}"));
        let d = collect(&sys, search);
        let notes: Vec<_> = d.user_dir.iter().map(|f| f.note.as_deref()).collect();
        assert_eq!(
            notes,
            [
                Some(
                    "bottle C; /home/<user>/.local/share/bottles/data.yml: larger than 1 MiB; \
                     /home/<user>/.local/share/bottles/bottles/C/placeholder.yml: larger than 1 MiB"
                ),
                Some("bottle A"),
            ]
        );
    }
    #[test]
    fn discover_with_runs_bottles_on_linux() {
        let sys = FakeSystem::linux(H).dir(&format!("{B}/bottles/PoE2/{UD}"));
        let d = discover_with(&sys);
        assert!(rows(&d.user_dir).iter().any(|r| r.1 == Source::Bottles));
    }
}
