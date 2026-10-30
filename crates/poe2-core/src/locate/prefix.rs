//! Probing Wine prefixes, and the plain-Wine and umu prefixes (docs/research/linux-paths.md Q3, Q8, Probe recipe 3–4); ADRs 0003, 0015.

use super::Source;
use super::candidates::Candidates;
use super::system::System;
use super::tpath::join;

/// PoE2's standalone install inside a prefix (linux-paths.md Q8).
const INSTALL: &str = "drive_c/Program Files (x86)/Grinding Gear Games/Path of Exile 2";

/// Probes `prefix` and `prefix/pfx` for PoE2's user dir and install; pushes every match. True if anything matched.
pub(crate) fn probe(
    sys: &impl System,
    prefix: &str,
    source: Source,
    note: &str,
    out: &mut Candidates,
) -> bool {
    let os = sys.target_os();
    let mut found = false;
    for base in [prefix.to_owned(), join(os, prefix, "pfx")] {
        let users = join(os, &base, "drive_c/users");
        for user in sys.list_dir(&users) {
            if user.eq_ignore_ascii_case("Public") {
                continue;
            }
            for docs in ["Documents", "My Documents"] {
                let dir = join(
                    os,
                    &users,
                    &format!("{user}/{docs}/My Games/Path of Exile 2"),
                );
                if sys.exists(&dir) {
                    out.user_dir(dir, source, Some(note.to_owned()));
                    found = true;
                }
            }
        }
        let installs = [
            INSTALL,
            "drive_c/Program Files/Grinding Gear Games/Path of Exile 2",
        ];
        for install in installs.map(|rel| join(os, &base, rel)) {
            if sys.exists(&install) {
                out.install(install, source, Some(note.to_owned()));
                found = true;
            }
        }
    }
    found
}

/// `probe`, then the would-be install `<prefix>/drive_c/Program Files (x86)/Grinding Gear Games/Path of Exile 2` if nothing matched (ADR 0015).
pub(crate) fn probe_default(
    sys: &impl System,
    prefix: &str,
    source: Source,
    note: &str,
    out: &mut Candidates,
) {
    if !probe(sys, prefix, source, note, out) {
        out.install(
            join(sys.target_os(), prefix, INSTALL),
            source,
            Some(note.to_owned()),
        );
    }
}

/// `$WINEPREFIX` (if absolute, ADR 0025), `~/.wine`, `~/Games/umu/*`.
pub(crate) fn search_wine(sys: &impl System, out: &mut Candidates) {
    let os = sys.target_os();
    if let Some(prefix) = sys.env_path("WINEPREFIX") {
        probe_default(sys, &prefix, Source::Fallback, "WINEPREFIX", out);
    }
    let Some(home) = sys.home_dir() else {
        return;
    };
    probe_default(
        sys,
        &join(os, &home, ".wine"),
        Source::Fallback,
        "default Wine prefix",
        out,
    );
    let umu = join(os, &home, "Games/umu");
    for name in sys.list_dir(&umu) {
        probe(
            sys,
            &join(os, &umu, &name),
            Source::Fallback,
            &format!("umu prefix {name}"),
            out,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::locate::{
        Source,
        candidates::collect,
        discover_with,
        fake::{FakeSystem, acf, rows},
        steam,
    };
    const H: &str = "/home/<user>";
    const CD: &str = "/home/<user>/.local/share/Steam/steamapps/compatdata";
    const UD: &str = "drive_c/users/steamuser/Documents/My Games/Path of Exile 2";
    const GGG: &str = "drive_c/Program Files (x86)/Grinding Gear Games/Path of Exile 2";
    fn steam_with_manifest() -> FakeSystem {
        FakeSystem::linux(H)
            .file(
                "/home/<user>/.local/share/Steam/steamapps/appmanifest_2694490.acf",
                &acf("Path of Exile 2"),
            )
            .dir("/home/<user>/.local/share/Steam/steamapps/common/Path of Exile 2")
    }
    #[test]
    fn store_prefix_user_dir_and_its_children() {
        let ud = format!("{CD}/2694490/pfx/{UD}");
        let d = collect(
            &steam_with_manifest().dir(&format!("{ud}/BuildPlanner")),
            steam::search_linux,
        );
        assert_eq!(
            rows(&d.user_dir),
            [(ud.as_str(), Source::ProtonPrefix, true)]
        );
        assert_eq!(
            d.user_dir[0].note.as_deref(),
            Some("compatdata/2694490 (store app)")
        );
        let bp = format!("{ud}/BuildPlanner");
        assert_eq!(
            rows(&d.build_planner),
            [(bp.as_str(), Source::ProtonPrefix, true)]
        );
    }
    #[test]
    fn a_store_prefix_not_created_yet_is_listed() {
        let ud = format!("{CD}/2694490/pfx/{UD}");
        let d = collect(&steam_with_manifest(), steam::search_linux);
        assert_eq!(
            rows(&d.user_dir),
            [(ud.as_str(), Source::ProtonPrefix, false)]
        );
        assert_eq!(
            d.user_dir[0].note.as_deref(),
            Some("compatdata/2694490 (store app); created on first launch")
        );
    }
    #[test]
    fn shortcut_prefixes_are_found_and_other_games_ignored() {
        let sys = FakeSystem::linux(H)
            .dir("/home/<user>/.local/share/Steam/steamapps")
            .dir(&format!("{CD}/2315204395/pfx/{UD}"))
            .file(&format!("{CD}/2315204395/pfx/{GGG}/logs/Client.txt"), "")
            .dir(&format!("{CD}/440/pfx/drive_c/users/steamuser/Documents"));
        let d = collect(&sys, steam::search_linux);
        let (ud, inst) = (
            format!("{CD}/2315204395/pfx/{UD}"),
            format!("{CD}/2315204395/pfx/{GGG}"),
        );
        assert_eq!(
            rows(&d.user_dir),
            [(ud.as_str(), Source::ProtonPrefix, true)]
        );
        assert_eq!(
            rows(&d.installs)[0],
            (inst.as_str(), Source::ProtonPrefix, true)
        );
        assert_eq!(
            d.installs[0].note.as_deref(),
            Some("compatdata/2315204395 (shortcut)")
        );
    }
    #[test]
    fn my_documents_links_and_public_add_no_user_dirs() {
        let users = format!("{CD}/2694490/pfx/drive_c/users");
        let sys = steam_with_manifest()
            .dir(&format!(
                "{users}/steamuser/Documents/My Games/Path of Exile 2"
            ))
            .link(
                &format!("{users}/steamuser/My Documents"),
                &format!("{users}/steamuser/Documents"),
            )
            .dir(&format!(
                "{users}/Public/Documents/My Games/Path of Exile 2"
            ));
        assert_eq!(collect(&sys, steam::search_linux).user_dir.len(), 1);
    }
    #[test]
    fn plain_wine_and_umu_prefixes() {
        let umu = "/home/<user>/Games/umu/umu-default/drive_c/users";
        let sys = FakeSystem::linux(H)
            .env("WINEPREFIX", "/home/<user>/wine/poe")
            .dir("/home/<user>/wine/poe/drive_c/users/<user>/Documents/My Games/Path of Exile 2")
            .dir(&format!("/home/<user>/wine/poe/{GGG}"))
            .dir(&format!(
                "{umu}/steamuser/Documents/My Games/Path of Exile 2"
            ))
            .link(&format!("{umu}/<user>"), &format!("{umu}/steamuser"));
        let d = collect(&sys, search_wine);
        let (wine, dflt) = (
            format!("/home/<user>/wine/poe/{GGG}"),
            format!("/home/<user>/.wine/{GGG}"),
        );
        assert_eq!(
            rows(&d.installs),
            [
                (wine.as_str(), Source::Fallback, true),
                (dflt.as_str(), Source::Fallback, false)
            ]
        );
        let umu_ud = format!("{umu}/steamuser/Documents/My Games/Path of Exile 2");
        assert_eq!(
            rows(&d.user_dir),
            [
                (
                    "/home/<user>/wine/poe/drive_c/users/<user>/Documents/My Games/Path of Exile 2",
                    Source::Fallback,
                    true
                ),
                (umu_ud.as_str(), Source::Fallback, true)
            ]
        );
        let notes: Vec<_> = d
            .installs
            .iter()
            .chain(&d.user_dir)
            .map(|f| f.note.as_deref())
            .collect();
        assert_eq!(
            notes,
            [
                Some("WINEPREFIX"),
                Some("default Wine prefix"),
                Some("WINEPREFIX"),
                Some("umu prefix umu-default")
            ]
        );
    }
    #[test]
    fn a_relative_wineprefix_is_ignored() {
        // A sentinel install at `/wine`, where FakeSystem resolves `wine` (ADR 0025).
        let sys = FakeSystem::linux("/home/<user>")
            .env("WINEPREFIX", "wine")
            .dir(&format!("/wine/{INSTALL}"));
        let d = collect(&sys, search_wine);
        assert!(
            d.installs
                .iter()
                .all(|f| f.note.as_deref() != Some("WINEPREFIX")),
            "{:?}",
            rows(&d.installs)
        );
    }
    #[test]
    fn discover_with_runs_steam_then_wine_on_linux() {
        let d = discover_with(&FakeSystem::linux(H));
        assert!(
            rows(&d.installs)
                .iter()
                .any(|r| r.0 == format!("/home/<user>/.wine/{GGG}"))
        );
    }
}
