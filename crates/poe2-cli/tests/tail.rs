//! `poe2 tail` (ADRs 0010, 0014 and 0016).
mod common;
use common::{TempDir, poe2};
use std::{
    io::{BufRead, BufReader, Read, Write},
    path::Path,
    process::{Child, Stdio},
    sync::mpsc,
    thread,
    time::Duration,
};
fn lines_of(r: impl Read + Send + 'static) -> mpsc::Receiver<String> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        for l in BufReader::new(r).lines() {
            if tx.send(l.unwrap()).is_err() {
                break;
            }
        }
    });
    rx
}
/// A running `poe2`, killed and reaped on drop so a failed assertion never leaks the follower.
struct Running(Child);
impl Drop for Running {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
fn spawn(home: &Path, args: &[&str]) -> (Running, mpsc::Receiver<String>, mpsc::Receiver<String>) {
    let mut c = poe2(home)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let (out, err) = (
        lines_of(c.stdout.take().unwrap()),
        lines_of(c.stderr.take().unwrap()),
    );
    (Running(c), out, err)
}
fn next(rx: &mpsc::Receiver<String>) -> String {
    rx.recv_timeout(Duration::from_secs(10))
        .expect("no output within 10 s")
}
fn append(p: &Path, s: &str) {
    std::fs::OpenOptions::new()
        .append(true)
        .open(p)
        .unwrap()
        .write_all(s.as_bytes())
        .unwrap();
}
#[test]
fn from_start_prints_existing_and_appended_lines_as_json() {
    let t = TempDir::new();
    let f = t.join("Client.txt");
    std::fs::write(
        &f,
        "2026/01/02 03:04:05 ***** LOG FILE OPENING *****\nhello\n",
    )
    .unwrap();
    let (_child, out, _err) = spawn(
        t.path(),
        &[
            "tail",
            "--log",
            f.to_str().unwrap(),
            "--from-start",
            "--json",
        ],
    );
    assert_eq!(
        next(&out),
        r#"{"timestamp":"2026-01-02T03:04:05","prefix":null,"event":{"type":"LogOpened"}}"#
    );
    assert_eq!(
        next(&out),
        r#"{"timestamp":null,"prefix":null,"event":{"type":"Unknown","raw":"hello"}}"#
    );
    append(&f, "@x\n");
    assert_eq!(
        next(&out),
        r#"{"timestamp":null,"prefix":null,"event":{"type":"Unknown","raw":"@x"}}"#
    );
}
#[test]
fn by_default_it_starts_at_the_end() {
    let t = TempDir::new();
    let f = t.join("Client.txt");
    std::fs::write(&f, "old\n").unwrap();
    let (_child, out, err) = spawn(t.path(), &["tail", "--log", f.to_str().unwrap()]);
    assert!(next(&err).starts_with("following "));
    append(&f, "new\n");
    assert_eq!(next(&out), "- Unknown raw=\"new\"");
}
#[test]
fn a_missing_log_exits_2() {
    let t = TempDir::new();
    let out = poe2(t.path())
        .args(["tail", "--log", t.join("none.txt").to_str().unwrap()])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("none.txt"));
}
#[cfg(unix)]
#[test]
fn no_client_log_exits_1_and_a_discovered_one_is_followed() {
    let home = TempDir::new();
    let out = poe2(home.path()).arg("tail").output().unwrap();
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stderr).contains("poe2 doctor"));
    common::steam_layout(home.path());
    let (_child, out, _err) = spawn(home.path(), &["tail", "--from-start"]);
    assert_eq!(next(&out), "- Unknown raw=\"x\"");
}
