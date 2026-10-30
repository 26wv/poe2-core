//! Corpus snapshots (ADR 0013): tests/corpus/<name>.json is exactly `poe2 parse <name>.txt --json`.
use poe2_core::log::read_lines;
use std::{
    fs,
    path::{Path, PathBuf},
};
const EVENTS: [&str; 10] = [
    "log_opened",
    "area_generated",
    "scene_set",
    "zone_entered",
    "level_up",
    "character_slain",
    "whisper_received",
    "whisper_sent",
    "afk_on",
    "afk_off",
];
fn corpus() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/corpus")
}
fn fixtures() -> Vec<PathBuf> {
    let mut v: Vec<_> = fs::read_dir(corpus())
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "txt"))
        .collect();
    v.sort();
    v
}
fn render(txt: &Path) -> String {
    let mut s = String::new();
    for line in read_lines(fs::File::open(txt).unwrap()) {
        s += &serde_json::to_string(&line.unwrap()).unwrap();
        s.push('\n');
    }
    s
}
fn camel(stem: &str) -> String {
    stem.split('_')
        .map(|w| {
            let mut c = w.chars();
            c.next()
                .map(|f| f.to_ascii_uppercase().to_string() + c.as_str())
                .unwrap_or_default()
        })
        .collect()
}
fn source_lines(txt: &Path) -> Vec<String> {
    String::from_utf8_lossy(&fs::read(txt).unwrap())
        .split('\n')
        .map(|l| {
            let l = l.strip_suffix('\r').unwrap_or(l);
            l.strip_prefix('\u{feff}').unwrap_or(l).to_owned()
        })
        .filter(|l| !l.is_empty())
        .collect()
}
#[test]
fn snapshots_match() {
    let update = std::env::var_os("UPDATE_SNAPSHOTS").is_some();
    let mut failures = Vec::new();
    for txt in fixtures() {
        let (actual, json) = (render(&txt), txt.with_extension("json"));
        if update {
            fs::write(&json, &actual).unwrap();
            continue;
        }
        let expected = fs::read_to_string(&json).unwrap_or_default();
        if let Some((i, (e, a))) = expected
            .lines()
            .zip(actual.lines())
            .enumerate()
            .find(|(_, (e, a))| e != a)
        {
            failures.push(format!(
                "{}:{}\n  expected {e}\n  actual   {a}",
                json.display(),
                i + 1
            ));
        } else if expected != actual {
            failures.push(format!(
                "{}: line count or final newline differs",
                json.display()
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "{}\nrun UPDATE_SNAPSHOTS=1 cargo test -p poe2-core --test corpus, then review the diff",
        failures.join("\n")
    );
}
#[test]
fn every_v1_event_has_a_fixture_and_every_fixture_a_snapshot() {
    for name in EVENTS.iter().chain(&["unknown", "session"]) {
        assert!(
            corpus().join(format!("{name}.txt")).is_file(),
            "{name}.txt missing"
        );
    }
    for txt in fixtures() {
        assert!(
            txt.with_extension("json").is_file(),
            "{} has no snapshot",
            txt.display()
        );
    }
}
#[test]
fn fixtures_hold_only_their_event() {
    for txt in fixtures()
        .into_iter()
        .filter(|t| !t.ends_with("session.txt"))
    {
        let want = camel(txt.file_stem().unwrap().to_str().unwrap());
        for line in read_lines(fs::File::open(&txt).unwrap()) {
            let v = serde_json::to_value(line.unwrap()).unwrap();
            assert_eq!(v["event"]["type"], want.as_str(), "{}: {v}", txt.display());
        }
    }
}
#[test]
fn unknown_lines_round_trip() {
    for txt in fixtures() {
        let src = source_lines(&txt);
        let got: Vec<_> = read_lines(fs::File::open(&txt).unwrap())
            .map(|l| serde_json::to_value(l.unwrap()).unwrap())
            .collect();
        assert_eq!(got.len(), src.len(), "{}", txt.display());
        for (v, s) in got.iter().zip(&src) {
            if v["event"]["type"] == "Unknown" {
                assert_eq!(v["event"]["raw"], s.as_str());
            }
        }
    }
}
