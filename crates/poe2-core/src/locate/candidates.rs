//! Candidates, de-duplication and ranking (ADR 0002 §4, §6–7; ADR 0015; docs/research/windows-paths.md Q6).

use super::system::{System, TargetOs};
use super::tpath::{join, key, normalize};
use super::{Discovery, Found, Source};
use std::cmp::Reverse;
use std::collections::HashSet;
use std::path::PathBuf;
use std::time::SystemTime;

/// Raw installs and user dirs as the searches find them, in discovery order.
#[derive(Default)]
pub(crate) struct Candidates {
    installs: Vec<Found<String>>,
    user_dirs: Vec<Found<String>>,
}

impl Candidates {
    pub(crate) fn install(&mut self, path: String, source: Source, note: Option<String>) {
        self.installs.push(raw(path, source, note));
    }

    pub(crate) fn user_dir(&mut self, path: String, source: Source, note: Option<String>) {
        self.user_dirs.push(raw(path, source, note));
    }
}

fn raw(value: String, source: Source, note: Option<String>) -> Found<String> {
    Found {
        value,
        source,
        exists: false,
        note,
    }
}

/// Resolves, de-duplicates and ranks the candidates, then derives the other three lists.
pub(crate) fn finish(sys: &impl System, c: Candidates) -> Discovery {
    let os = sys.target_os();
    let log = |dir: &str| join(os, dir, "logs/Client.txt");
    let installs = settle(sys, c.installs, |f| sys.modified(&log(&f.value)));
    let user_dirs = settle(sys, c.user_dirs, |_| None);
    let client_log = installs.iter().map(|f| at(f, log(&f.value))).collect();
    let build_planner = user_dirs
        .iter()
        .map(|f| at(f, join(os, &f.value, "BuildPlanner")))
        .collect();
    let filters = user_dirs
        .iter()
        .flat_map(|f| [f.clone(), at(f, child(sys, &f.value, "OnlineFilters"))])
        .collect();
    Discovery {
        client_log: into_paths(settle(sys, client_log, |f| sys.modified(&f.value))),
        build_planner: into_paths(settle(sys, build_planner, |_| None)),
        filters: into_paths(settle(sys, filters, |_| None)),
        installs: into_paths(installs),
        user_dir: into_paths(user_dirs),
    }
}

/// A candidate derived from `parent`, keeping its Source and note.
fn at(parent: &Found<String>, value: String) -> Found<String> {
    Found {
        value,
        ..parent.clone()
    }
}

/// Resolves each candidate, keeps the first of each `key`, then ranks (ADR 0002 §6–7):
/// env overrides, then existing ones, then the newest `mtime`, then discovery order.
fn settle(
    sys: &impl System,
    list: Vec<Found<String>>,
    mtime: impl Fn(&Found<String>) -> Option<SystemTime>,
) -> Vec<Found<String>> {
    let os = sys.target_os();
    let mut seen = HashSet::new();
    let mut list: Vec<Found<String>> = list
        .into_iter()
        .map(|mut f| {
            f.value = normalize(os, &f.value);
            f.exists = sys.exists(&f.value);
            if os == TargetOs::Linux && f.exists {
                f.value = sys.canonical(&f.value).unwrap_or(f.value);
            }
            f
        })
        .filter(|f| seen.insert(key(os, &f.value)))
        .collect();
    list.sort_by_cached_key(|f| {
        (
            f.source != Source::EnvOverride,
            !f.exists,
            Reverse(mtime(f)),
        )
    });
    list
}

fn into_paths(list: Vec<Found<String>>) -> Vec<Found<PathBuf>> {
    list.into_iter()
        .map(|f| Found {
            value: PathBuf::from(f.value),
            source: f.source,
            exists: f.exists,
            note: f.note,
        })
        .collect()
}

/// `dir`'s child `name`. On Linux an existing child whose name matches ignoring ASCII case wins,
/// an exact match first.
pub(crate) fn child(sys: &impl System, dir: &str, name: &str) -> String {
    let os = sys.target_os();
    if os == TargetOs::Linux {
        let entries = sys.list_dir(dir);
        let found = entries
            .iter()
            .find(|e| *e == name)
            .or_else(|| entries.iter().find(|e| e.eq_ignore_ascii_case(name)));
        if let Some(entry) = found {
            return join(os, dir, entry);
        }
    }
    join(os, dir, name)
}

/// Runs `search` on fresh candidates, then [`finish`].
#[cfg(test)]
pub(crate) fn collect<S: System>(sys: &S, search: impl FnOnce(&S, &mut Candidates)) -> Discovery {
    let mut c = Candidates::default();
    search(sys, &mut c);
    finish(sys, c)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::locate::{
        Found, Source, discover_with,
        fake::{FakeSystem, rows},
    };
    use std::path::PathBuf;
    fn paths(list: &[Found<PathBuf>]) -> Vec<&str> {
        rows(list).into_iter().map(|r| r.0).collect()
    }

    #[test]
    fn env_overrides_come_first_and_derive_their_children() {
        let sys = FakeSystem::linux("/home/<user>")
            .env("POE2_INSTALL_DIR", "/games/poe2/")
            .file("/games/poe2/logs/Client.txt", "")
            .env("POE2_USER_DIR", "/data/poe2")
            .dir("/data/poe2/BuildPlanner")
            .dir("/data/poe2/onlinefilters");
        let d = discover_with(&sys);
        assert_eq!(
            rows(&d.installs)[0],
            ("/games/poe2", Source::EnvOverride, true)
        );
        assert_eq!(d.installs[0].note.as_deref(), Some("POE2_INSTALL_DIR"));
        assert_eq!(
            rows(&d.client_log)[0],
            ("/games/poe2/logs/Client.txt", Source::EnvOverride, true)
        );
        assert_eq!(
            rows(&d.user_dir)[0],
            ("/data/poe2", Source::EnvOverride, true)
        );
        assert_eq!(
            rows(&d.build_planner)[0],
            ("/data/poe2/BuildPlanner", Source::EnvOverride, true)
        );
        assert_eq!(
            rows(&d.filters)[..2],
            [
                ("/data/poe2", Source::EnvOverride, true),
                ("/data/poe2/onlinefilters", Source::EnvOverride, true)
            ]
        );
    }
    #[test]
    fn windows_env_override_is_normalised_and_listed_even_when_missing() {
        let d = discover_with(&FakeSystem::windows().env("POE2_INSTALL_DIR", "D:/Games/PoE2/"));
        assert_eq!(
            rows(&d.installs)[0],
            (r"D:\Games\PoE2", Source::EnvOverride, false)
        );
        assert_eq!(rows(&d.client_log)[0].0, r"D:\Games\PoE2\logs\Client.txt");
    }
    #[test]
    fn ranking_env_then_existing_then_newest_client_log() {
        let sys = FakeSystem::linux("/h")
            .dir("/a")
            .file_mtime("/b/logs/Client.txt", "", 200)
            .file_mtime("/c/logs/Client.txt", "", 300);
        let mut c = Candidates::default();
        for (p, s) in [
            ("/missing", Source::Steam),
            ("/a", Source::Lutris),
            ("/b", Source::Heroic),
            ("/c", Source::Bottles),
            ("/env", Source::EnvOverride),
        ] {
            c.install(p.to_owned(), s, None);
        }
        let d = finish(&sys, c);
        assert_eq!(paths(&d.installs), ["/env", "/c", "/b", "/a", "/missing"]);
        assert_eq!(
            paths(&d.client_log),
            [
                "/env/logs/Client.txt",
                "/c/logs/Client.txt",
                "/b/logs/Client.txt",
                "/a/logs/Client.txt",
                "/missing/logs/Client.txt"
            ]
        );
    }
    #[test]
    fn linux_duplicates_merge_by_canonical_path_keeping_the_first() {
        let sys = FakeSystem::linux("/h")
            .dir("/h/.local/share/Steam/x")
            .link("/h/.steam/steam", "/h/.local/share/Steam");
        let mut c = Candidates::default();
        c.install(
            "/h/.steam/steam/x".to_owned(),
            Source::Steam,
            Some("first".to_owned()),
        );
        c.install(
            "/h/.local/share/Steam/x".to_owned(),
            Source::FlatpakSteam,
            None,
        );
        c.install("/h/nope".to_owned(), Source::Steam, None);
        c.install("/h/nope/".to_owned(), Source::Steam, None);
        let d = finish(&sys, c);
        assert_eq!(
            rows(&d.installs),
            [
                ("/h/.local/share/Steam/x", Source::Steam, true),
                ("/h/nope", Source::Steam, false)
            ]
        );
        assert_eq!(d.installs[0].note.as_deref(), Some("first"));
    }
    #[test]
    fn windows_duplicates_merge_ignoring_case_and_are_never_canonicalised() {
        let sys = FakeSystem::windows().dir(r"C:\Games\PoE2");
        let mut c = Candidates::default();
        c.install(r"C:\Games\PoE2".to_owned(), Source::Standalone, None);
        c.install("c:/games/poe2/".to_owned(), Source::Fallback, None);
        assert_eq!(
            rows(&finish(&sys, c).installs),
            [(r"C:\Games\PoE2", Source::Standalone, true)]
        );
    }
    #[test]
    fn missing_online_filters_is_listed_as_written() {
        let mut c = Candidates::default();
        c.user_dir("/u".to_owned(), Source::Lutris, None);
        let d = finish(&FakeSystem::linux("/h").dir("/u"), c);
        assert_eq!(
            rows(&d.filters),
            [
                ("/u", Source::Lutris, true),
                ("/u/OnlineFilters", Source::Lutris, false)
            ]
        );
    }
    #[test]
    fn discovery_serialises_to_the_documented_json() {
        let d = Discovery {
            installs: vec![Found {
                value: PathBuf::from("/x"),
                source: Source::Steam,
                exists: true,
                note: None,
            }],
            ..Default::default()
        };
        assert_eq!(
            serde_json::to_string(&d).unwrap(),
            r#"{"installs":[{"value":"/x","source":"Steam","exists":true,"note":null}],"client_log":[],"user_dir":[],"build_planner":[],"filters":[]}"#
        );
    }
    #[test]
    fn collect_runs_the_search_and_an_exact_child_wins() {
        let sys = FakeSystem::linux("/h")
            .dir("/u/ONLINEFILTERS")
            .dir("/u/OnlineFilters");
        let d = collect(&sys, |_, c| {
            c.user_dir("/u".to_owned(), Source::Bottles, None)
        });
        assert_eq!(paths(&d.filters), ["/u", "/u/OnlineFilters"]);
    }
}
