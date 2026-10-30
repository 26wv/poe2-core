//! The `System` trait that all locator probing goes through (ADR 0002 §1–3; ADR 0011).

use super::tpath::absolute_or_unc;
use std::io;
use std::time::SystemTime;

/// Launcher file reads stop here (ADR 0002 consequences).
pub(crate) const READ_CAP: u64 = 1024 * 1024;

/// The error a read over [`READ_CAP`] fails with.
pub(crate) fn too_large() -> io::Error {
    io::Error::new(io::ErrorKind::FileTooLarge, "larger than 1 MiB")
}

/// The operating system whose layout a System presents.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TargetOs {
    Windows,
    Linux,
}

/// A registry hive: HKCU or HKLM.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Hive {
    CurrentUser,
    LocalMachine,
}

/// Every read of the environment, the registry or the filesystem the locator makes.
/// Paths are target paths built with `tpath`.
pub(crate) trait System {
    fn target_os(&self) -> TargetOs;
    /// `None` if unset, empty or not UTF-8.
    fn env(&self, name: &str) -> Option<String>;
    /// `env(name)` normalised, if it is an absolute path (UNC allowed, ADR 0024); a relative value
    /// counts as unset, never resolved against the working directory (ADR 0025).
    fn env_path(&self, name: &str) -> Option<String> {
        absolute_or_unc(self.target_os(), &self.env(name)?)
    }
    fn home_dir(&self) -> Option<String>;
    /// The Windows Documents known folder, unverified so a dangling redirect still comes back
    /// (ADR 0006). `None` on Linux.
    fn documents_folder(&self) -> Option<String>;
    /// A REG_SZ value; `None` on Linux.
    fn registry_string(&self, hive: Hive, subkey: &str, value: &str) -> Option<String>;
    /// Follows links.
    fn exists(&self, path: &str) -> bool;
    /// Lossy UTF-8; `NotFound` if missing; `InvalidInput` if not a regular file (a FIFO or device
    /// is never opened); [`too_large`] over [`READ_CAP`].
    fn read(&self, path: &str) -> io::Result<String>;
    /// UTF-8 entry names, sorted; empty if missing.
    fn list_dir(&self, path: &str) -> Vec<String>;
    fn modified(&self, path: &str) -> Option<SystemTime>;
    /// Links resolved; `None` if missing.
    fn canonical(&self, path: &str) -> Option<String>;
}
