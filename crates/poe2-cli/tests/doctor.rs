mod common;
use common::{TempDir, poe2};

/// Runs on every OS. On Windows `discover()` reads the real registry and Documents folder,
/// so there it checks only the shape and the exit code.
#[test]
fn doctor_without_poe2_exits_0_and_lists_what_it_checked() {
    let home = TempDir::new();
    let out = poe2(home.path()).arg("doctor").output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    let text = String::from_utf8(out.stdout).unwrap();
    assert!(text.starts_with("KIND"), "{text}");
    if cfg!(unix) {
        assert!(
            text.lines().filter(|l| l.contains("  no  ")).count() >= 3,
            "{text}"
        );
        assert!(
            text.contains("Steam root not found") && text.contains("0 of "),
            "{text}"
        );
    }
    let json = poe2(home.path())
        .args(["doctor", "--json"])
        .output()
        .unwrap();
    assert_eq!(json.status.code(), Some(0));
    assert!(
        serde_json::from_slice::<serde_json::Value>(&json.stdout).unwrap()["installs"].is_array()
    );
}

#[cfg(unix)]
#[test]
fn doctor_finds_a_synthetic_steam_install() {
    let home = TempDir::new();
    common::steam_layout(home.path());
    let text = String::from_utf8(poe2(home.path()).arg("doctor").output().unwrap().stdout).unwrap();
    let row = text
        .lines()
        .find(|l| l.starts_with("installs") && l.contains("yes"))
        .unwrap();
    assert!(
        row.contains("Steam")
            && row.contains("steamapps/common/Path of Exile 2")
            && row.contains("app manifest in library"),
        "{row}"
    );
}
