//! Target paths built as text (ADR 0002 §5–6; docs/research/windows-paths.md Q2).

use super::system::TargetOs;

fn sep(os: TargetOs) -> char {
    match os {
        TargetOs::Windows => '\\',
        TargetOs::Linux => '/',
    }
}

/// Windows: `/` becomes `\`. Both: trailing separators are trimmed, except on a root (`X:\`, `/`).
pub(crate) fn normalize(os: TargetOs, path: &str) -> String {
    let path = match os {
        TargetOs::Windows => path.replace('/', "\\"),
        TargetOs::Linux => path.to_owned(),
    };
    let trimmed = path.trim_end_matches(sep(os));
    if trimmed.len() == path.len() {
        return path;
    }
    let is_root = match os {
        TargetOs::Windows => trimmed.is_empty() || trimmed.len() == 2 && trimmed.ends_with(':'),
        TargetOs::Linux => trimmed.is_empty(),
    };
    if is_root {
        format!("{trimmed}{}", sep(os))
    } else {
        trimmed.to_owned()
    }
}

/// `path` normalised, if it is a local absolute path: `X:\…` on Windows, `/…` on Linux. An empty or
/// relative path from launcher data is skipped, never resolved against the working directory
/// (ADR 0020), and so is a `\\…` path (UNC, `\\?\…`, `\\.\pipe\…`): probing one would make Windows
/// open a network connection or a device (ADR 0024).
pub(crate) fn absolute(os: TargetOs, path: &str) -> Option<String> {
    let path = normalize(os, path);
    let absolute = match os {
        TargetOs::Windows => {
            matches!(path.as_bytes(), [d, b':', b'\\', ..] if d.is_ascii_alphabetic())
        }
        TargetOs::Linux => path.starts_with('/'),
    };
    absolute.then_some(path)
}

/// `absolute`, or on Windows also a `\\…` path: for values the user or the OS sets, not launcher
/// data (ADR 0024).
pub(crate) fn absolute_or_unc(os: TargetOs, path: &str) -> Option<String> {
    let path = normalize(os, path);
    match os {
        TargetOs::Windows if path.starts_with(r"\\") => Some(path),
        _ => absolute(os, &path),
    }
}

/// `base`, one separator, then `rel` (whose `/` become `\` on Windows).
pub(crate) fn join(os: TargetOs, base: &str, rel: &str) -> String {
    let mut out = normalize(os, base);
    if !out.ends_with(sep(os)) {
        out.push(sep(os));
    }
    match os {
        TargetOs::Windows => out.push_str(&rel.replace('/', "\\")),
        TargetOs::Linux => out.push_str(rel),
    }
    out
}

/// A path's identity for de-duplication: Windows compares case-insensitively.
pub(crate) fn key(os: TargetOs, path: &str) -> String {
    let path = normalize(os, path);
    match os {
        TargetOs::Windows => path.to_lowercase(),
        TargetOs::Linux => path,
    }
}

/// `~` and `~/…` become paths under `home`; anything else is returned unchanged.
pub(crate) fn expand_tilde(home: &str, path: &str) -> String {
    if path == "~" {
        home.to_owned()
    } else if let Some(rest) = path.strip_prefix("~/") {
        format!("{}/{rest}", home.trim_end_matches('/'))
    } else {
        path.to_owned()
    }
}

/// A Flatpak app's sandbox path as seen on the host (ADR 0004): `<home>/<rel>` becomes
/// `<home>/.var/app/<app_id>/<rel>` and `/var/data/<rel>` becomes `<home>/.var/app/<app_id>/data/<rel>`.
/// Paths already in the app's folder, and host paths such as `/mnt`, give `None`.
pub(crate) fn flatpak_remap(home: &str, app_id: &str, path: &str) -> Option<String> {
    let os = TargetOs::Linux;
    let app = join(os, home, &format!(".var/app/{app_id}"));
    if under(path, &app).is_some() {
        return None;
    }
    if let Some(rel) = under(path, &normalize(os, home)) {
        return Some(join(os, &app, rel));
    }
    under(path, "/var/data").map(|rel| join(os, &app, &format!("data/{rel}")))
}

/// `path` below `dir`, matching whole components: the rest without its leading `/`.
fn under<'a>(path: &'a str, dir: &str) -> Option<&'a str> {
    match path.strip_prefix(dir)? {
        "" => Some(""),
        rest => rest.strip_prefix('/'),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use TargetOs::{Linux, Windows};

    #[test]
    fn windows_join_and_normalize() {
        assert_eq!(
            join(
                Windows,
                "c:/program files (x86)/steam",
                "steamapps/libraryfolders.vdf"
            ),
            r"c:\program files (x86)\steam\steamapps\libraryfolders.vdf"
        );
        assert_eq!(join(Windows, r"C:\", "Games"), r"C:\Games");
        assert_eq!(
            normalize(Windows, r"D:\Games\Path of Exile 2\"),
            r"D:\Games\Path of Exile 2"
        );
        assert_eq!(normalize(Windows, "C:/"), r"C:\");
    }

    #[test]
    fn absolute_keeps_local_rooted_paths_only() {
        assert_eq!(absolute(Windows, "d:/Games/").as_deref(), Some(r"d:\Games"));
        assert_eq!(absolute(Linux, "/mnt/x/").as_deref(), Some("/mnt/x"));
        for p in [
            "",
            "C:",
            r"\Games",
            r"Games\PoE2",
            r"\\192.0.2.66\s\PoE2",
            "//192.0.2.66/s/PoE2",
            r"\\?\C:\PoE2",
            r"\\?\UNC\192.0.2.66\s",
            r"\\.\pipe\poe2",
            r"\\.\COM1",
        ] {
            assert_eq!(absolute(Windows, p), None, "{p}");
        }
        for p in ["", "rel/x", "~/x", "./x"] {
            assert_eq!(absolute(Linux, p), None, "{p}");
        }
    }

    #[test]
    fn linux_join_and_normalize() {
        assert_eq!(join(Linux, "/home/<user>/", "a/b"), "/home/<user>/a/b");
        assert_eq!(join(Linux, "/", "mnt"), "/mnt");
        assert_eq!(normalize(Linux, "/a/b//"), "/a/b");
        assert_eq!(normalize(Linux, "/"), "/");
    }

    #[test]
    fn keys_fold_case_and_separators_only_on_windows() {
        assert_eq!(
            key(Windows, "C:/Program Files (x86)/Steam/"),
            key(Windows, r"c:\program files (x86)\steam")
        );
        assert_eq!(
            key(Windows, r"C:\Users\<user>\Mes Jeux\ÉTÉ"),
            key(Windows, r"c:\users\<user>\mes jeux\été")
        );
        assert_ne!(key(Linux, "/a/Steam"), key(Linux, "/a/steam"));
    }

    #[test]
    fn expands_a_leading_tilde_only() {
        assert_eq!(
            expand_tilde("/home/<user>", "~/Games/x"),
            "/home/<user>/Games/x"
        );
        assert_eq!(expand_tilde("/home/<user>", "~"), "/home/<user>");
        assert_eq!(expand_tilde("/home/<user>", "/abs/~"), "/abs/~");
        assert_eq!(expand_tilde("/home/<user>", "~other/x"), "~other/x");
    }

    #[test]
    fn flatpak_remap_rules() {
        let (h, app) = ("/home/<user>", "com.valvesoftware.Steam");
        assert_eq!(
            flatpak_remap(h, app, "/home/<user>/.local/share/Steam").as_deref(),
            Some("/home/<user>/.var/app/com.valvesoftware.Steam/.local/share/Steam")
        );
        assert_eq!(
            flatpak_remap(h, app, "/var/data/SteamLibrary").as_deref(),
            Some("/home/<user>/.var/app/com.valvesoftware.Steam/data/SteamLibrary")
        );
        for p in [
            "/mnt/games/x",
            "/media/x",
            "/run/media/<user>/x",
            "/home/<user>/.var/app/com.valvesoftware.Steam/x",
            "/home/<user>2/x",
            "/opt/x",
        ] {
            assert_eq!(flatpak_remap(h, app, p), None, "{p}");
        }
    }
}
