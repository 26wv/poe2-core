//! An in-memory `System` for tests (ADR 0002 §3; ADR 0011; docs/research/linux-paths.md Q1, Q8).

use super::steam::APP_ID;
use super::system::{Hive, READ_CAP, System, TargetOs, too_large};
use super::tpath::{join, key, normalize};
use super::{Found, Source};
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::io;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// More link hops than this count as missing, as on Linux.
const MAX_HOPS: u32 = 40;

enum Node {
    File { contents: String, mtime: u64 },
    Dir,
    Link(String),
}

/// A chosen target OS's files, links, registry and environment, held in memory.
pub(crate) struct FakeSystem {
    os: TargetOs,
    env: HashMap<String, String>,
    home: Option<String>,
    documents: Option<String>,
    registry: HashMap<(Hive, String, String), String>,
    /// By `key`, with each path as written.
    nodes: BTreeMap<String, (String, Node)>,
    /// How often each path (by `key`) was listed.
    listed: RefCell<HashMap<String, usize>>,
}

impl FakeSystem {
    fn new(os: TargetOs) -> Self {
        FakeSystem {
            os,
            env: HashMap::new(),
            home: None,
            documents: None,
            registry: HashMap::new(),
            nodes: BTreeMap::new(),
            listed: RefCell::default(),
        }
    }

    /// Target Linux; `HOME` and `home_dir()` are `home`.
    pub(crate) fn linux(home: &str) -> Self {
        let mut sys = Self::new(TargetOs::Linux).env("HOME", home);
        sys.home = Some(home.to_owned());
        sys
    }

    /// Target Windows; no environment, no documents folder.
    pub(crate) fn windows() -> Self {
        Self::new(TargetOs::Windows)
    }

    pub(crate) fn env(mut self, name: &str, value: &str) -> Self {
        self.env.insert(name.to_owned(), value.to_owned());
        self
    }

    pub(crate) fn documents(mut self, path: &str) -> Self {
        self.documents = Some(path.to_owned());
        self
    }

    pub(crate) fn reg(mut self, hive: Hive, subkey: &str, value: &str, data: &str) -> Self {
        self.registry.insert(
            (hive, subkey.to_lowercase(), value.to_lowercase()),
            data.to_owned(),
        );
        self
    }

    /// A file with modified time 0 s.
    pub(crate) fn file(self, path: &str, contents: &str) -> Self {
        self.file_mtime(path, contents, 0)
    }

    pub(crate) fn file_mtime(self, path: &str, contents: &str, secs: u64) -> Self {
        let node = Node::File {
            contents: contents.to_owned(),
            mtime: secs,
        };
        self.add(path, node)
    }

    pub(crate) fn dir(self, path: &str) -> Self {
        self.add(path, Node::Dir)
    }

    /// `target` is absolute.
    pub(crate) fn link(self, path: &str, target: &str) -> Self {
        self.add(path, Node::Link(target.to_owned()))
    }

    /// How many times `list_dir` was called on `path`.
    pub(crate) fn list_count(&self, path: &str) -> usize {
        self.listed
            .borrow()
            .get(&key(self.os, path))
            .copied()
            .unwrap_or(0)
    }

    /// Adds `node` at `path` and any missing ancestors as dirs.
    fn add(mut self, path: &str, node: Node) -> Self {
        let (mut at, parts) = self.split(path);
        for part in parts {
            self.nodes
                .entry(key(self.os, &at))
                .or_insert_with(|| (at.clone(), Node::Dir));
            at = join(self.os, &at, &part);
        }
        self.nodes.insert(key(self.os, &at), (at, node));
        self
    }

    /// A normalised path's root (`/`, `X:\`) and its components.
    fn split(&self, path: &str) -> (String, Vec<String>) {
        let path = normalize(self.os, path);
        let sep = match self.os {
            TargetOs::Windows => '\\',
            TargetOs::Linux => '/',
        };
        let mut parts = path.split(sep).filter(|p| !p.is_empty()).map(str::to_owned);
        let root = match self.os {
            TargetOs::Windows => format!("{}\\", parts.next().unwrap_or_default()),
            TargetOs::Linux => "/".to_owned(),
        };
        (root, parts.collect())
    }

    /// The node `path` leads to, links resolved component by component.
    fn resolve(&self, path: &str) -> Option<(&String, &Node)> {
        let (root, parts) = self.split(path);
        let mut pending: Vec<String> = parts.into_iter().rev().collect();
        let (mut at, mut node) = self.get(&root)?;
        let mut hops = 0;
        while let Some(part) = pending.pop() {
            if !matches!(node, Node::Dir) {
                return None;
            }
            let (written, next) = self.get(&join(self.os, at, &part))?;
            if let Node::Link(target) = next {
                hops += 1;
                if hops > MAX_HOPS {
                    return None;
                }
                let (root, parts) = self.split(target);
                pending.extend(parts.into_iter().rev());
                (at, node) = self.get(&root)?;
            } else {
                (at, node) = (written, next);
            }
        }
        Some((at, node))
    }

    fn get(&self, path: &str) -> Option<(&String, &Node)> {
        self.nodes
            .get(&key(self.os, path))
            .map(|(written, node)| (written, node))
    }
}

impl System for FakeSystem {
    fn target_os(&self) -> TargetOs {
        self.os
    }

    fn env(&self, name: &str) -> Option<String> {
        self.env.get(name).filter(|v| !v.is_empty()).cloned()
    }

    fn home_dir(&self) -> Option<String> {
        self.home.clone()
    }

    fn documents_folder(&self) -> Option<String> {
        self.documents.clone()
    }

    fn registry_string(&self, hive: Hive, subkey: &str, value: &str) -> Option<String> {
        self.registry
            .get(&(hive, subkey.to_lowercase(), value.to_lowercase()))
            .cloned()
    }

    fn exists(&self, path: &str) -> bool {
        self.resolve(path).is_some()
    }

    fn read(&self, path: &str) -> io::Result<String> {
        match self.resolve(path) {
            Some((_, Node::File { contents, .. })) if contents.len() as u64 > READ_CAP => {
                Err(too_large())
            }
            Some((_, Node::File { contents, .. })) => Ok(contents.clone()),
            Some(_) => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "not a regular file",
            )),
            None => Err(io::ErrorKind::NotFound.into()),
        }
    }

    fn list_dir(&self, path: &str) -> Vec<String> {
        *self
            .listed
            .borrow_mut()
            .entry(key(self.os, path))
            .or_default() += 1;
        let Some((dir, Node::Dir)) = self.resolve(path) else {
            return Vec::new();
        };
        let dir = key(self.os, dir);
        let mut names: Vec<String> = self
            .nodes
            .values()
            .filter_map(|(written, _)| {
                let (root, mut parts) = self.split(written);
                let name = parts.pop()?;
                let parent = parts.iter().fold(root, |at, part| join(self.os, &at, part));
                (key(self.os, &parent) == dir).then_some(name)
            })
            .collect();
        names.sort();
        names
    }

    fn modified(&self, path: &str) -> Option<SystemTime> {
        match self.resolve(path)? {
            (_, Node::File { mtime, .. }) => Some(UNIX_EPOCH + Duration::from_secs(*mtime)),
            _ => None,
        }
    }

    fn canonical(&self, path: &str) -> Option<String> {
        self.resolve(path).map(|(written, _)| written.clone())
    }
}

/// Each candidate's path, source and `exists`, for terse assertions.
pub(crate) fn rows(list: &[Found<PathBuf>]) -> Vec<(&str, Source, bool)> {
    list.iter()
        .map(|f| (f.value.to_str().unwrap(), f.source, f.exists))
        .collect()
}

/// A current-layout library list with `paths` as libraries `"0"`, `"1"`, …, backslashes doubled.
pub(crate) fn vdf(paths: &[&str]) -> String {
    let libraries: String = paths
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let p = p.replace('\\', r"\\");
            format!("\t\"{i}\"\n\t{{\n\t\t\"path\"\t\t\"{p}\"\n\t}}\n")
        })
        .collect();
    format!("\"libraryfolders\"\n{{\n{libraries}}}\n")
}

/// PoE2's app manifest with `installdir` written as given.
pub(crate) fn acf(installdir: &str) -> String {
    format!(r#""AppState" {{ "appid" "{APP_ID}" "installdir" "{installdir}" }}"#)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::locate::system::{Hive, READ_CAP, System};
    use std::{
        io::ErrorKind,
        time::{Duration, UNIX_EPOCH},
    };

    #[test]
    fn windows_paths_ignore_case_and_slash_style() {
        let sys = FakeSystem::windows().file(r"C:\Steam\steamapps\libraryfolders.vdf", "x");
        assert!(sys.exists("c:/steam/STEAMAPPS/libraryfolders.vdf"));
        assert_eq!(
            sys.read(r"c:\STEAM\steamapps\LIBRARYFOLDERS.VDF").unwrap(),
            "x"
        );
        assert_eq!(sys.list_dir(r"c:\steam"), ["steamapps"]);
    }

    #[test]
    fn listing_is_sorted_and_empty_when_missing() {
        let sys = FakeSystem::linux("/home/<user>")
            .file("/d/b", "")
            .file("/d/a", "")
            .dir("/d/c")
            .dir("/d/망명자");
        assert_eq!(sys.list_dir("/d"), ["a", "b", "c", "망명자"]);
        assert!(sys.list_dir("/nope").is_empty());
    }

    #[test]
    fn reads_fail_for_missing_oversized_and_dirs() {
        let big = "x".repeat(READ_CAP as usize + 1);
        let sys = FakeSystem::linux("/h").file("/big", &big).dir("/d");
        assert_eq!(
            sys.read("/missing").unwrap_err().kind(),
            ErrorKind::NotFound
        );
        let err = sys.read("/big").unwrap_err();
        assert_eq!(
            (err.kind(), err.to_string()),
            (ErrorKind::FileTooLarge, "larger than 1 MiB".to_owned())
        );
        let err = sys.read("/d").unwrap_err();
        assert_eq!(
            (err.kind(), err.to_string()),
            (ErrorKind::InvalidInput, "not a regular file".to_owned())
        );
    }

    #[test]
    fn links_resolve_and_loops_end() {
        let sys = FakeSystem::linux("/home/<user>")
            .dir("/home/<user>/.local/share/Steam/steamapps")
            .link(
                "/home/<user>/.steam/steam",
                "/home/<user>/.local/share/Steam",
            )
            .link("/a", "/b")
            .link("/b", "/a")
            .link("/dangling", "/nowhere");
        assert!(sys.exists("/home/<user>/.steam/steam/steamapps"));
        assert_eq!(
            sys.canonical("/home/<user>/.steam/steam/steamapps")
                .as_deref(),
            Some("/home/<user>/.local/share/Steam/steamapps")
        );
        assert_eq!(sys.list_dir("/home/<user>/.steam/steam"), ["steamapps"]);
        assert!(!sys.exists("/a/x") && sys.canonical("/a").is_none());
        assert!(!sys.exists("/dangling") && sys.canonical("/dangling").is_none());
    }

    #[test]
    fn environment_registry_and_times() {
        // By reference: on an owned FakeSystem, `.env(name)` would pick the builder `env`.
        let lin = &FakeSystem::linux("/home/<user>")
            .env("EMPTY", "")
            .file_mtime("/f", "", 100);
        assert_eq!(lin.env("HOME").as_deref(), Some("/home/<user>"));
        assert_eq!(lin.home_dir().as_deref(), Some("/home/<user>"));
        assert_eq!(
            (lin.env("EMPTY"), lin.env("UNSET"), lin.documents_folder()),
            (None, None, None)
        );
        assert_eq!(
            lin.modified("/f"),
            Some(UNIX_EPOCH + Duration::from_secs(100))
        );
        assert_eq!(lin.modified("/missing"), None);
        let win = FakeSystem::windows()
            .documents(r"C:\Users\<user>\OneDrive\Documents")
            .reg(
                Hive::CurrentUser,
                r"Software\Valve\Steam",
                "SteamPath",
                "c:/steam",
            );
        assert_eq!(
            win.registry_string(Hive::CurrentUser, r"SOFTWARE\valve\steam", "steampath")
                .as_deref(),
            Some("c:/steam")
        );
        assert_eq!(
            win.registry_string(Hive::LocalMachine, r"Software\Valve\Steam", "SteamPath"),
            None
        );
        assert_eq!(
            win.documents_folder().as_deref(),
            Some(r"C:\Users\<user>\OneDrive\Documents")
        );
        assert!(!win.exists(r"C:\Users\<user>\OneDrive\Documents")); // dangling redirect: listed, not created
    }
}
