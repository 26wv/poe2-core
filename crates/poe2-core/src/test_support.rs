//! Test-only helpers shared by the unit tests.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(0);

/// A fresh empty folder under the system temp dir, removed on drop.
pub(crate) struct TempDir(PathBuf);

impl TempDir {
    pub(crate) fn new() -> TempDir {
        let n = NEXT.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!("poe2-core-test-{}-{n}", std::process::id()));
        let _ = std::fs::remove_dir_all(&path); // a stale copy from an earlier run with the same pid
        std::fs::create_dir_all(&path).unwrap();
        TempDir(path)
    }

    pub(crate) fn path(&self) -> &Path {
        &self.0
    }

    pub(crate) fn join(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// A FIFO at `path`, made with the `mkfifo` tool (std has no call for it).
#[cfg(unix)]
pub(crate) fn mkfifo(path: &Path) {
    let status = std::process::Command::new("mkfifo")
        .arg(path)
        .status()
        .unwrap();
    assert!(status.success(), "mkfifo failed");
}

#[cfg(unix)] // only the FIFO tests use it
/// `f`'s result, or a panic after 5 s, so a call that blocks fails its test instead of hanging it.
pub(crate) fn within_5s<T: Send + 'static>(f: impl FnOnce() -> T + Send + 'static) -> T {
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || tx.send(f()));
    rx.recv_timeout(std::time::Duration::from_secs(5))
        .expect("blocked for 5 s")
}

#[cfg(test)]
mod tests {
    use super::TempDir;
    #[test]
    fn temp_dirs_are_unique_empty_and_removed_on_drop() {
        let (a, b) = (TempDir::new(), TempDir::new());
        assert_ne!(a.path(), b.path());
        assert_eq!(std::fs::read_dir(a.path()).unwrap().count(), 0);
        std::fs::write(a.join("f.txt"), "x").unwrap();
        let kept = a.path().to_path_buf();
        drop(a);
        assert!(!kept.exists());
    }
}
