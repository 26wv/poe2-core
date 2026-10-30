mod common;
use common::{TempDir, poe2};

/// Runs on every OS. On Windows `discover()` reads the real registry and Documents folder,
/// so this checks only the shape and that the exit code matches it, never specific paths.
#[test]
fn json_has_the_discovery_shape_and_the_exit_code_matches_it() {
    let home = TempDir::new();
    let out = poe2(home.path())
        .args(["paths", "--json"])
        .output()
        .unwrap();
    let text = String::from_utf8(out.stdout).unwrap();
    assert!(
        text.ends_with("}\n") && text.contains("\n  \"installs\": ["),
        "pretty JSON"
    );
    let v: serde_json::Value = serde_json::from_str(&text).unwrap();
    let kinds = [
        "installs",
        "client_log",
        "user_dir",
        "build_planner",
        "filters",
    ];
    let keys: Vec<&str> = v.as_object().unwrap().keys().map(String::as_str).collect();
    assert_eq!(keys.len(), kinds.len());
    let mut any = false;
    for k in kinds {
        for c in v[k].as_array().unwrap() {
            let c = c.as_object().unwrap();
            assert_eq!(c.len(), 4, "{k}");
            assert!(c["value"].is_string() && c["source"].is_string(), "{k}");
            assert!(c["note"].is_string() || c["note"].is_null(), "{k}");
            any |= c["exists"].as_bool().unwrap();
        }
    }
    let want = Some(if any { 0 } else { 1 });
    assert_eq!(out.status.code(), want);
    let plain = poe2(home.path()).arg("paths").output().unwrap();
    assert_eq!(plain.status.code(), want);
    assert_eq!(plain.stdout.is_empty(), !any);
}

#[cfg(unix)]
#[test]
fn nothing_found_exits_1_and_still_prints_the_json() {
    let home = TempDir::new();
    let out = poe2(home.path())
        .args(["paths", "--json"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    for k in [
        "installs",
        "client_log",
        "user_dir",
        "build_planner",
        "filters",
    ] {
        assert!(v[k].is_array(), "{k}");
    }
    let plain = poe2(home.path()).arg("paths").output().unwrap();
    assert_eq!(
        (plain.status.code(), plain.stdout.is_empty()),
        (Some(1), true)
    );
}

#[cfg(unix)]
#[test]
fn a_synthetic_steam_install_is_printed() {
    let home = TempDir::new();
    let log = common::steam_layout(home.path());
    let out = poe2(home.path()).arg("paths").output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    let want = format!(
        "client_log\t{}\n",
        std::fs::canonicalize(&log).unwrap().display()
    );
    assert!(String::from_utf8(out.stdout).unwrap().contains(&want));
}
