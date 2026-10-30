mod common;
use common::run;
use std::{fs, path::Path};

#[test]
fn parse_json_matches_every_corpus_snapshot() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/corpus");
    let mut n = 0;
    for p in fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "txt"))
    {
        n += 1;
        let (code, out, err) = run(&["poe2", "parse", p.to_str().unwrap(), "--json"]);
        assert_eq!((code, err.as_str()), (0, ""), "{p:?}");
        assert_eq!(
            out,
            fs::read_to_string(p.with_extension("json")).unwrap(),
            "{p:?}"
        );
    }
    assert!(n >= 12);
}

#[test]
fn a_missing_file_exits_2() {
    let (code, out, err) = run(&["poe2", "parse", "/nonexistent/poe2/Client.txt"]);
    assert_eq!(code, 2);
    assert!(out.is_empty() && err.contains("Client.txt"));
}

#[cfg(unix)]
#[test]
fn a_closed_stdout_ends_parse_quietly() {
    use common::TempDir;
    use std::io::BufRead;
    use std::process::{Command, Stdio};
    let t = TempDir::new();
    let f = t.join("big.txt");
    fs::write(
        &f,
        "2026/01/02 03:04:05 ***** LOG FILE OPENING *****\n".repeat(200_000),
    )
    .unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_poe2"))
        .args(["parse", f.to_str().unwrap(), "--json"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut first = String::new();
    std::io::BufReader::new(child.stdout.take().unwrap())
        .read_line(&mut first)
        .unwrap(); // reader dropped: pipe closed
    let out = child.wait_with_output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    assert!(
        out.stderr.is_empty(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
}
