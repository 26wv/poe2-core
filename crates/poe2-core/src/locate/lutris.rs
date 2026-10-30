//! Lutris's recorded and default Wine prefixes (docs/research/linux-paths.md Q6, Q9, Conflicting); ADRs 0003, 0005, 0015, 0018.

use super::Source;
use super::candidates::Candidates;
use super::prefix::{probe, probe_default};
use super::system::System;
use super::tpath::{expand_tilde, join};
use super::yaml::scalar;
use std::io::ErrorKind;

/// The top-level `game.prefix` of every `games/*.yml` in the old and new config dirs, unfiltered
/// by game (ADR 0003), then the PoE2 installer's default prefix, noted with any read errors (ADR 0015).
pub(crate) fn search(sys: &impl System, out: &mut Candidates) {
    let os = sys.target_os();
    let Some(home) = sys.home_dir() else {
        return;
    };
    let xdg = |name: &str, default: &str| {
        sys.env(name)
            .filter(|dir| dir.starts_with('/'))
            .unwrap_or_else(|| join(os, &home, default))
    };
    let dirs = [
        xdg("XDG_CONFIG_HOME", ".config"),
        xdg("XDG_DATA_HOME", ".local/share"),
    ];
    let mut errors = String::new();
    for dir in dirs {
        let games = join(os, &dir, "lutris/games");
        for name in sys.list_dir(&games) {
            if !name.ends_with(".yml") {
                continue;
            }
            let file = join(os, &games, &name);
            let text = match sys.read(&file) {
                Ok(text) => text,
                Err(e) if e.kind() == ErrorKind::NotFound => continue,
                Err(e) => {
                    errors.push_str(&format!("; {file}: {e}"));
                    continue;
                }
            };
            // Untrusted: an empty or relative prefix is skipped, never resolved against the cwd.
            let Some(prefix) = scalar(&text, &["game", "prefix"])
                .map(|p| expand_tilde(&home, &p))
                .filter(|p| p.starts_with('/'))
            else {
                continue;
            };
            probe(
                sys,
                &prefix,
                Source::Lutris,
                &format!("Lutris games/{name}"),
                out,
            );
        }
    }
    probe_default(
        sys,
        &join(os, &home, "Games/path-of-exile-2"),
        Source::Lutris,
        &format!("Lutris default prefix{errors}"),
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
    const DATA: &str = "/home/<user>/.local/share/lutris/games";
    const CFG: &str = "/home/<user>/.config/lutris/games";
    const GGG: &str = "drive_c/Program Files (x86)/Grinding Gear Games/Path of Exile 2";
    const UD: &str = "drive_c/users/<user>/Documents/My Games/Path of Exile 2";
    #[test]
    fn game_prefixes_from_both_config_dirs() {
        let sys = FakeSystem::linux(H)
            .file(
                &format!("{DATA}/path-of-exile-2-1700000000.yml"),
                include_str!("fixtures/lutris_plain.yml"),
            )
            .dir(&format!("/home/<user>/Games/path-of-exile-2/{GGG}"))
            .file(
                &format!("{CFG}/poe2-copy-1700000001.yml"),
                include_str!("fixtures/lutris_quoted.yml"),
            )
            .dir("/home/<user>/Games/PoE #2's prefix/drive_c/users/<user>/Documents/My Games/Path of Exile 2");
        let d = collect(&sys, search);
        let inst = format!("/home/<user>/Games/path-of-exile-2/{GGG}");
        assert_eq!(rows(&d.installs), [(inst.as_str(), Source::Lutris, true)]);
        assert_eq!(
            d.installs[0].note.as_deref(),
            Some("Lutris games/path-of-exile-2-1700000000.yml")
        );
        assert_eq!(
            rows(&d.user_dir),
            [(
                "/home/<user>/Games/PoE #2's prefix/drive_c/users/<user>/Documents/My Games/Path of Exile 2",
                Source::Lutris,
                true
            )]
        );
        assert_eq!(
            d.user_dir[0].note.as_deref(),
            Some("Lutris games/poe2-copy-1700000001.yml")
        );
    }
    #[test]
    fn script_prefixes_are_ignored_and_the_default_is_listed() {
        let sys = FakeSystem::linux(H).file(
            &format!("{DATA}/x.yml"),
            include_str!("fixtures/lutris_script_only.yml"),
        );
        let d = collect(&sys, search);
        let row = format!("/home/<user>/Games/path-of-exile-2/{GGG}");
        assert_eq!(rows(&d.installs), [(row.as_str(), Source::Lutris, false)]);
        assert_eq!(d.installs[0].note.as_deref(), Some("Lutris default prefix"));
    }
    #[test]
    fn a_tilde_prefix_is_expanded() {
        let sys = FakeSystem::linux(H)
            .file(&format!("{DATA}/t.yml"), "game:\n  prefix: ~/Games/poe\n")
            .dir(&format!("/home/<user>/Games/poe/{GGG}"));
        let inst = format!("/home/<user>/Games/poe/{GGG}");
        assert_eq!(
            rows(&collect(&sys, search).installs)[0],
            (inst.as_str(), Source::Lutris, true)
        );
    }
    #[test]
    fn empty_relative_and_non_yml_entries_are_skipped() {
        // FakeSystem resolves "" to `/` and `rel/poe2` to `/rel/poe2`, so both would match if kept;
        // `yaml::scalar` already drops the empty one, the absolute-path filter drops the relative one.
        let sys = FakeSystem::linux(H)
            .file(&format!("{DATA}/a.yml"), "game:\n  prefix: ''\n")
            .dir(&format!("/{UD}"))
            .file(&format!("{DATA}/b.yml"), "game:\n  prefix: rel/poe2\n")
            .dir(&format!("/rel/poe2/{UD}"))
            .file(&format!("{DATA}/c.yaml"), "game:\n  prefix: /p\n")
            .dir(&format!("/p/{UD}"));
        assert!(collect(&sys, search).user_dir.is_empty());
    }
    #[test]
    fn xdg_dirs_are_honoured() {
        let sys = FakeSystem::linux(H)
            .env("XDG_CONFIG_HOME", "/cfg")
            .env("XDG_DATA_HOME", "/data")
            .file("/cfg/lutris/games/a.yml", "game:\n  prefix: /p\n")
            .dir(&format!("/p/{UD}"))
            .file("/data/lutris/games/b.yml", "game:\n  prefix: /q\n")
            .dir(&format!("/q/{UD}"));
        let d = collect(&sys, search);
        let notes: Vec<_> = d.user_dir.iter().map(|f| f.note.as_deref()).collect();
        assert_eq!(
            notes,
            [Some("Lutris games/a.yml"), Some("Lutris games/b.yml")]
        );
    }
    #[test]
    fn relative_xdg_dirs_fall_back_to_the_defaults() {
        let sys = FakeSystem::linux(H)
            .env("XDG_CONFIG_HOME", "cfg")
            .env("XDG_DATA_HOME", "data")
            .file("/cfg/lutris/games/a.yml", "game:\n  prefix: /q\n")
            .file("/data/lutris/games/b.yml", "game:\n  prefix: /q\n")
            .dir(&format!("/q/{UD}"))
            .file(&format!("{CFG}/c.yml"), "game:\n  prefix: /p\n")
            .file(&format!("{DATA}/d.yml"), "game:\n  prefix: /r\n")
            .dir(&format!("/p/{UD}"))
            .dir(&format!("/r/{UD}"));
        let d = collect(&sys, search);
        let notes: Vec<_> = d.user_dir.iter().map(|f| f.note.as_deref()).collect();
        assert_eq!(
            notes,
            [Some("Lutris games/c.yml"), Some("Lutris games/d.yml")]
        );
    }
    #[test]
    fn bom_and_unreadable_files() {
        let sys = FakeSystem::linux(H)
            .file(&format!("{CFG}/a.yml"), "\u{feff}game:\n  prefix: /p\n")
            .dir(&format!("/p/{UD}"))
            .file(
                &format!("{DATA}/big.yml"),
                &"x".repeat(READ_CAP as usize + 1),
            );
        let d = collect(&sys, search);
        assert_eq!(d.user_dir.len(), 1);
        assert_eq!(
            d.installs[0].note.as_deref(),
            Some(
                "Lutris default prefix; /home/<user>/.local/share/lutris/games/big.yml: larger than 1 MiB"
            )
        );
    }
    #[test]
    fn discover_with_runs_lutris_on_linux() {
        let d = discover_with(&FakeSystem::linux(H));
        let row = format!("/home/<user>/Games/path-of-exile-2/{GGG}");
        assert!(rows(&d.installs).iter().any(|r| r.0 == row));
    }
}
