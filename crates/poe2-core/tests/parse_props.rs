//! Guard properties for `parse_line` (ADR 0008 §1–2 and §7).

use poe2_core::log::{Event, parse_line};
use proptest::prelude::*;

fn normalise(s: &str) -> &str {
    let s = s.strip_suffix('\n').unwrap_or(s);
    let s = s.strip_suffix('\r').unwrap_or(s);
    s.strip_prefix('\u{feff}').unwrap_or(s)
}

proptest! {
    #[test] fn never_panics(s in any::<String>()) { let _ = parse_line(&s); }
    #[test] fn prefixed_bodies_never_panic(body in any::<String>()) { let _ = parse_line(&format!("2026/01/02 03:04:05 1 a [INFO Client 1] {body}")); }
    #[test] fn blank_iff_none_and_unknown_raw_round_trips(s in "[^\n]*") {
        let got = parse_line(&s);
        prop_assert_eq!(got.is_none(), normalise(&s).is_empty());
        if let Some(Event::Unknown { raw, .. }) = got.map(|l| l.event) { prop_assert_eq!(raw, normalise(&s)); }
    }
}

const PIECES: &[&str] = &[
    "@From ",
    "@To ",
    ": ",
    " (",
    ") is now level ",
    " has been slain.",
    "Generating level ",
    " area \"",
    "\" with seed ",
    "[SCENE] Set Source [",
    "]",
    "AFK mode is now ON",
    ". Autoreply \"",
    "\"",
    ".",
    "망",
];
proptest! {
    #[test] fn fragment_soup_never_panics(parts in proptest::collection::vec(
        prop_oneof![proptest::sample::select(PIECES).prop_map(str::to_owned), "[0-9]{1,25}"], 0..10)) {
        let _ = parse_line(&format!("2026/01/02 03:04:05 1 a [INFO Client 1] {}", parts.concat()));
    }
}
