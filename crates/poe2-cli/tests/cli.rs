mod common;
use common::run;

#[test]
fn help_and_version_exit_0() {
    let (code, out, _) = run(&["poe2", "--help"]);
    assert_eq!(code, 0);
    for cmd in ["doctor", "paths", "tail", "parse"] {
        assert!(out.contains(cmd), "{cmd} missing from --help");
    }
    assert_eq!(
        run(&["poe2", "--version"]),
        (0, "poe2 0.1.0\n".to_owned(), String::new())
    );
}

#[test]
fn usage_errors_exit_2_with_a_message() {
    for args in [
        &["poe2"][..],
        &["poe2", "bogus"],
        &["poe2", "parse"],
        &["poe2", "tail", "--log"],
        &["poe2", "doctor", "--nope"],
    ] {
        let (code, out, err) = run(args);
        assert_eq!(code, 2, "{args:?}");
        assert!(out.is_empty() && !err.is_empty(), "{args:?}");
    }
}
