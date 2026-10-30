//! Locator: finds Path of Exile 2's install folder and `Client.txt` on Windows and Linux (ADR 0002 §4; ADR 0011).

mod bottles;
mod candidates;
#[cfg(test)]
mod fake;
mod heroic;
mod json;
mod keyvalues;
mod lutris;
mod prefix;
mod real;
mod steam;
mod system;
mod tpath;
mod windows;
mod yaml;

use candidates::{Candidates, finish};
use serde::Serialize;
use std::path::PathBuf;
use system::{System, TargetOs};

/// The ranked candidates from one locator run, one list per kind of location.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Discovery {
    /// Folders holding a copy of the game, with its executable and `logs/` folder.
    pub installs: Vec<Found<PathBuf>>,
    /// `logs/Client.txt` inside each install.
    pub client_log: Vec<Found<PathBuf>>,
    /// The game's per-user folders, `My Games/Path of Exile 2` inside a Documents folder.
    pub user_dir: Vec<Found<PathBuf>>,
    /// The `BuildPlanner` folder in each user dir.
    pub build_planner: Vec<Found<PathBuf>>,
    /// Folders the game reads item filters from: each user dir and its `OnlineFilters` child.
    pub filters: Vec<Found<PathBuf>>,
}

/// A candidate: one possible location and how it was found.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Found<T> {
    /// The location.
    pub value: T,
    /// Where the candidate came from.
    pub source: Source,
    /// Whether the location existed when probed.
    pub exists: bool,
    /// How the candidate was found, when that needs saying.
    pub note: Option<String>,
}

/// Where a candidate came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[non_exhaustive]
pub enum Source {
    /// `POE2_INSTALL_DIR` or `POE2_USER_DIR`.
    EnvOverride,
    /// A Steam root and its libraries.
    Steam,
    /// A Proton prefix in a Steam library.
    ProtonPrefix,
    /// The Flatpak Steam root and its libraries.
    FlatpakSteam,
    /// A standalone install.
    Standalone,
    /// The Epic Games Launcher's manifests.
    Epic,
    /// Heroic's records.
    Heroic,
    /// Lutris's records.
    Lutris,
    /// Bottles' records.
    Bottles,
    /// The Windows Documents known folder.
    KnownFolder,
    /// A documented default location, not taken from any launcher's records.
    Fallback,
}

/// Finds every candidate location for Path of Exile 2 on this machine, ranked: env overrides
/// first, then candidates that exist, then the most recently written client log.
pub fn discover() -> Discovery {
    discover_with(&real::RealSystem)
}

/// The env overrides, then the searches for `sys`'s target OS (ADR 0002 §4).
pub(crate) fn discover_with(sys: &impl System) -> Discovery {
    let os = sys.target_os();
    let mut c = Candidates::default();
    if let Some(dir) = sys.env_path("POE2_INSTALL_DIR") {
        c.install(
            dir,
            Source::EnvOverride,
            Some("POE2_INSTALL_DIR".to_owned()),
        );
    }
    if let Some(dir) = sys.env_path("POE2_USER_DIR") {
        c.user_dir(dir, Source::EnvOverride, Some("POE2_USER_DIR".to_owned()));
    }
    match os {
        TargetOs::Windows => {
            steam::search_windows(sys, &mut c);
            windows::search(sys, &mut c);
        }
        TargetOs::Linux => {
            steam::search_linux(sys, &mut c);
            heroic::search(sys, &mut c);
            lutris::search(sys, &mut c);
            bottles::search(sys, &mut c);
            prefix::search_wine(sys, &mut c);
        }
    }
    finish(sys, c)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relative_overrides_are_ignored_and_unc_ones_kept() {
        use crate::locate::fake::{FakeSystem, rows};
        // Sentinels where FakeSystem resolves the relative values (ADR 0025).
        let linux = FakeSystem::linux("/home/<user>")
            .env("POE2_INSTALL_DIR", "games/poe2")
            .env("POE2_USER_DIR", "data/poe2")
            .dir("/games/poe2")
            .dir("/data/poe2");
        let windows = FakeSystem::windows()
            .env("POE2_INSTALL_DIR", r"Games\PoE2")
            .env("POE2_USER_DIR", r"Data\PoE2")
            .dir(r"Games\PoE2")
            .dir(r"Data\PoE2");
        for d in [discover_with(&linux), discover_with(&windows)] {
            for list in [&d.installs, &d.user_dir] {
                assert!(
                    list.iter().all(|f| f.source != Source::EnvOverride),
                    "{:?}",
                    rows(list)
                );
            }
        }
        // An override is the user's own choice, so it may name a share (ADR 0024).
        let share = FakeSystem::windows()
            .env("POE2_INSTALL_DIR", r"\\192.0.2.66\s\PoE2")
            .env("POE2_USER_DIR", "//192.0.2.66/s/My Games/Path of Exile 2");
        let d = discover_with(&share);
        assert_eq!(
            (rows(&d.installs)[0], rows(&d.user_dir)[0]),
            (
                (r"\\192.0.2.66\s\PoE2", Source::EnvOverride, false),
                (
                    r"\\192.0.2.66\s\My Games\Path of Exile 2",
                    Source::EnvOverride,
                    false
                )
            )
        );
    }
    #[test]
    fn every_source_is_reachable() {
        use crate::locate::{
            fake::{FakeSystem, acf},
            system::Hive,
        };
        let windows = FakeSystem::windows()
            .env("POE2_INSTALL_DIR", r"D:\PoE2")
            .reg(
                Hive::CurrentUser,
                r"Software\Valve\Steam",
                "SteamPath",
                "c:/steam",
            )
            .reg(
                Hive::CurrentUser,
                r"Software\GrindingGearGames\Path of Exile 2",
                "InstallLocation",
                r"D:\GGG\Path of Exile 2",
            )
            .file(
                r"C:\ProgramData\Epic\EpicGamesLauncher\Data\Manifests\A.item",
                r#"{"CatalogNamespace": "58a18be6bbba41a0bf52b014bb81d33b", "InstallLocation": "D:\\Epic\\PathOfExile2"}"#,
            )
            .documents(r"C:\Users\<user>\Documents");
        let linux = FakeSystem::linux("/home/<user>")
            .env("POE2_USER_DIR", "/data/poe2")
            .file(
                "/home/<user>/.local/share/Steam/steamapps/appmanifest_2694490.acf",
                &acf("Path of Exile 2"),
            )
            .dir("/home/<user>/.local/share/bottles/bottles/PoE2/drive_c/users/<user>/Documents/My Games/Path of Exile 2");
        let mut seen = std::collections::HashSet::new();
        for d in [discover_with(&windows), discover_with(&linux)] {
            for list in [
                &d.installs,
                &d.client_log,
                &d.user_dir,
                &d.build_planner,
                &d.filters,
            ] {
                seen.extend(list.iter().map(|f| f.source));
            }
        }
        for s in [
            Source::EnvOverride,
            Source::Steam,
            Source::ProtonPrefix,
            Source::FlatpakSteam,
            Source::Standalone,
            Source::Epic,
            Source::Heroic,
            Source::Lutris,
            Source::Bottles,
            Source::KnownFolder,
            Source::Fallback,
        ] {
            assert!(seen.contains(&s), "{s:?} never appears");
        }
    }
}
