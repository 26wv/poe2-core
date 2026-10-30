//! Helpers shared by the CLI integration tests.
#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(0);

// copied from crates/poe2-core/src/test_support.rs
/// A fresh empty folder under the system temp dir, removed on drop.
pub struct TempDir(PathBuf);

impl TempDir {
    pub fn new() -> TempDir {
        let n = NEXT.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!("poe2-cli-test-{}-{n}", std::process::id()));
        let _ = std::fs::remove_dir_all(&path); // a stale copy from an earlier run with the same pid
        std::fs::create_dir_all(&path).unwrap();
        TempDir(path)
    }

    pub fn path(&self) -> &Path {
        &self.0
    }

    pub fn join(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Runs `poe2_cli::run` in-process: (exit code, stdout, stderr).
pub fn run(args: &[&str]) -> (u8, String, String) {
    let (mut out, mut err) = (Vec::new(), Vec::new());
    let code = poe2_cli::run(args, &mut out, &mut err);
    (
        code,
        String::from_utf8(out).unwrap(),
        String::from_utf8(err).unwrap(),
    )
}

/// The poe2 binary with POE2_* removed; on Unix the environment is cleared except HOME = home.
pub fn poe2(home: &Path) -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_poe2"));
    if cfg!(unix) {
        cmd.env_clear();
    } else {
        for (key, _) in std::env::vars_os() {
            if key
                .to_string_lossy()
                .to_ascii_uppercase()
                .starts_with("POE2_")
            {
                cmd.env_remove(key);
            }
        }
    }
    cmd.env("HOME", home);
    cmd
}

/// A native Steam install of PoE2 under `home`; returns its client log path (holds one line, "x\n").
pub fn steam_layout(home: &Path) -> PathBuf {
    let apps = home.join(".local/share/Steam/steamapps");
    let logs = apps.join("common/Path of Exile 2/logs");
    std::fs::create_dir_all(&logs).unwrap();
    std::fs::write(
        apps.join("appmanifest_2694490.acf"),
        r#""AppState" { "appid" "2694490" "installdir" "Path of Exile 2" }"#,
    )
    .unwrap();
    let log = logs.join("Client.txt");
    std::fs::write(&log, "x\n").unwrap();
    log
}
