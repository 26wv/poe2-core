//! The real system: probes the host machine (ADR 0002 §2; ADR 0006; ADR 0011).

#[cfg(windows)]
mod win32;

use super::system::{Hive, READ_CAP, System, TargetOs, too_large};
use std::fs;
use std::io::{self, Read};
use std::path::Path;
use std::time::SystemTime;

/// The host machine; any non-Windows host counts as Linux.
pub(crate) struct RealSystem;

impl System for RealSystem {
    fn target_os(&self) -> TargetOs {
        if cfg!(windows) {
            TargetOs::Windows
        } else {
            TargetOs::Linux
        }
    }

    fn env(&self, name: &str) -> Option<String> {
        std::env::var(name).ok().filter(|v| !v.is_empty())
    }

    fn home_dir(&self) -> Option<String> {
        std::env::home_dir()?.into_os_string().into_string().ok()
    }

    fn documents_folder(&self) -> Option<String> {
        #[cfg(windows)]
        return win32::documents_folder();
        #[cfg(not(windows))]
        None
    }

    fn registry_string(&self, hive: Hive, subkey: &str, value: &str) -> Option<String> {
        #[cfg(windows)]
        return win32::registry_string(hive, subkey, value);
        #[cfg(not(windows))]
        {
            let _ = (hive, subkey, value);
            None
        }
    }

    fn exists(&self, path: &str) -> bool {
        Path::new(path).exists()
    }

    fn read(&self, path: &str) -> io::Result<String> {
        // Opening a FIFO or a device blocks or has side effects, so only regular files are opened.
        // Known limit: the path can be swapped between this check and the open; open with O_NONBLOCK if that matters.
        if !fs::metadata(path)?.is_file() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "not a regular file",
            ));
        }
        let mut bytes = Vec::new();
        fs::File::open(path)?
            .take(READ_CAP + 1)
            .read_to_end(&mut bytes)?;
        if bytes.len() as u64 > READ_CAP {
            return Err(too_large());
        }
        Ok(String::from_utf8_lossy(&bytes).into_owned())
    }

    fn list_dir(&self, path: &str) -> Vec<String> {
        let Ok(entries) = fs::read_dir(path) else {
            return Vec::new();
        };
        let mut names: Vec<String> = entries
            .filter_map(|e| e.ok()?.file_name().into_string().ok())
            .collect();
        names.sort();
        names
    }

    fn modified(&self, path: &str) -> Option<SystemTime> {
        fs::metadata(path).ok()?.modified().ok()
    }

    fn canonical(&self, path: &str) -> Option<String> {
        fs::canonicalize(path)
            .ok()?
            .into_os_string()
            .into_string()
            .ok()
    }
}

#[cfg(test)]
mod tests {
    use super::RealSystem;
    use crate::locate::system::{Hive, READ_CAP, System, TargetOs};
    use crate::test_support::TempDir;
    use std::{fs, io::ErrorKind};

    fn s(p: std::path::PathBuf) -> String {
        p.to_str().unwrap().to_owned()
    }

    #[test]
    fn reads_with_the_cap_and_lossy_decoding() {
        let t = TempDir::new();
        fs::write(t.join("a.txt"), b"a\xffb").unwrap();
        fs::write(t.join("big"), vec![b'x'; READ_CAP as usize + 1]).unwrap();
        assert_eq!(RealSystem.read(&s(t.join("a.txt"))).unwrap(), "a\u{fffd}b");
        assert_eq!(
            RealSystem.read(&s(t.join("none"))).unwrap_err().kind(),
            ErrorKind::NotFound
        );
        assert_eq!(
            RealSystem.read(&s(t.join("big"))).unwrap_err().kind(),
            ErrorKind::FileTooLarge
        );
    }

    #[test]
    fn lists_sorted_names_and_reports_times() {
        let t = TempDir::new();
        for n in ["b", "a", "c"] {
            fs::write(t.join(n), "").unwrap();
        }
        assert_eq!(RealSystem.list_dir(&s(t.join(""))), ["a", "b", "c"]);
        assert!(RealSystem.list_dir(&s(t.join("missing"))).is_empty());
        assert!(
            RealSystem.exists(&s(t.join("a"))) && RealSystem.modified(&s(t.join("a"))).is_some()
        );
        assert!(
            !RealSystem.exists(&s(t.join("zz"))) && RealSystem.modified(&s(t.join("zz"))).is_none()
        );
    }

    #[cfg(unix)]
    #[test]
    fn reads_only_regular_files() {
        use crate::test_support::{mkfifo, within_5s};
        let t = TempDir::new();
        mkfifo(&t.join("fifo"));
        std::os::unix::fs::symlink(t.join("fifo"), t.join("link")).unwrap();
        fs::create_dir(t.join("dir")).unwrap();
        for name in ["fifo", "link", "dir"] {
            let path = s(t.join(name));
            let err = within_5s(move || RealSystem.read(&path)).unwrap_err();
            assert_eq!(
                (err.kind(), err.to_string()),
                (ErrorKind::InvalidInput, "not a regular file".to_owned()),
                "{name}"
            );
        }
    }

    #[cfg(unix)]
    #[test]
    fn links_non_utf8_names_and_loops() {
        use std::os::unix::{ffi::OsStrExt, fs::symlink};
        let t = TempDir::new();
        fs::create_dir(t.join("real")).unwrap();
        symlink(t.join("real"), t.join("link")).unwrap();
        symlink(t.join("loop2"), t.join("loop1")).unwrap();
        symlink(t.join("loop1"), t.join("loop2")).unwrap();
        fs::write(t.path().join(std::ffi::OsStr::from_bytes(b"\xff")), "").unwrap();
        assert_eq!(
            RealSystem.canonical(&s(t.join("link"))),
            RealSystem.canonical(&s(t.join("real")))
        );
        assert!(
            RealSystem.canonical(&s(t.join("loop1"))).is_none()
                && !RealSystem.exists(&s(t.join("loop1")))
        );
        assert_eq!(
            RealSystem.list_dir(&s(t.join(""))),
            ["link", "loop1", "loop2", "real"]
        );
    }

    #[test]
    fn target_os_and_platform_reads() {
        assert_eq!(
            RealSystem.target_os(),
            if cfg!(windows) {
                TargetOs::Windows
            } else {
                TargetOs::Linux
            }
        );
        assert_eq!(RealSystem.env("POE2_CORE_SURELY_UNSET_7F3A"), None);
        if !cfg!(windows) {
            assert_eq!(
                RealSystem.registry_string(Hive::CurrentUser, r"Software\Valve\Steam", "SteamPath"),
                None
            );
            assert_eq!(RealSystem.documents_folder(), None);
        }
    }
}
