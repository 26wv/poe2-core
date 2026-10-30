//! The event a log line's body carries (docs/research/client-log.md "Body shapes", "Event details", Conflicting 1–3; ADRs 0007, 0008, 0009, 0012).

use serde::Serialize;

use super::locale::{ENGLISH, match_any, match_template};

/// The typed meaning of a log line: one of the v1 events (ADR 0007), or an unknown line.
///
/// Serialises internally tagged, as `{"type":"<Variant>",…fields}`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "type")]
#[non_exhaustive]
pub enum Event {
    /// The `***** LOG FILE OPENING *****` line the game writes at each launch.
    LogOpened,
    /// An engine line recording a new area instance: `Generating level <lvl> area "<area id>" with seed <seed>`.
    #[non_exhaustive]
    AreaGenerated {
        /// The area level.
        area_level: u32,
        /// The internal area id, as written.
        area_id: String,
        /// The generation seed.
        seed: u64,
    },
    /// An engine line naming the area now shown: `[SCENE] Set Source [<area name>]`.
    #[non_exhaustive]
    SceneSet {
        /// The area name, verbatim, including `(null)` (loading) and `(unknown)` (login, character select).
        area_name: String,
    },
    /// The system message `: You have entered <area name>.`; not every load writes it.
    #[non_exhaustive]
    ZoneEntered {
        /// The localised area name, without the final `.`.
        area_name: String,
    },
    /// A system message that a character (yours or a party member's) reached a new character level.
    #[non_exhaustive]
    LevelUp {
        /// The character name.
        name: String,
        /// The character class, which can be several words.
        class: String,
        /// The new character level.
        level: u32,
    },
    /// A system message that a character (yours or a party member's) has died.
    #[non_exhaustive]
    CharacterSlain {
        /// The character name.
        name: String,
    },
    /// A whisper from another player: `@From <name>: <text>`.
    #[non_exhaustive]
    WhisperReceived {
        /// The sender's name, up to the first `: `.
        sender: String,
        /// The whisper text.
        text: String,
    },
    /// A whisper to another player: `@To <name>: <text>`.
    #[non_exhaustive]
    WhisperSent {
        /// The recipient's name, up to the first `: `.
        recipient: String,
        /// The whisper text.
        text: String,
    },
    /// The system message that away-from-keyboard mode was switched on.
    #[non_exhaustive]
    AfkOn {
        /// The autoreply text, if the line carries one.
        autoreply: Option<String>,
    },
    /// The system message that away-from-keyboard mode was switched off.
    AfkOff,
    /// A non-blank line that matches no v1 event (ADR 0008).
    #[non_exhaustive]
    Unknown {
        /// The whole line text, prefix included, after one trailing `\n`, one trailing `\r` and one leading BOM are removed (ADR 0008 §1).
        raw: String,
    },
}

/// The engine line for a new area instance; English in every client (ADR 0009).
const AREA_GENERATED: &str = "Generating level {level} area \"{area_id}\" with seed {seed}";
/// The engine line naming the area now shown; English in every client (ADR 0009).
const SCENE_SET: &str = "[SCENE] Set Source [{area}]";

/// The v1 event a prefixed line's body matches, or `None` when none fits.
///
/// Engine lines are tried first and matched outside the locale table (ADR 0009). A body after
/// `: ` is a system message and never falls through to the whisper templates.
pub(crate) fn match_body(body: &str) -> Option<Event> {
    area_generated(body)
        .or_else(|| area_generated(body.strip_prefix(": ")?))
        .or_else(|| scene_set(body))
        .or_else(|| match body.strip_prefix(": ") {
            Some(text) => system_message(text),
            None => whisper(body),
        })
}

/// Matches a whisper against the English locale table; the name runs up to the first `: ` (ADR 0007).
fn whisper(body: &str) -> Option<Event> {
    if let Some(c) = match_any(ENGLISH.whisper_received, body) {
        return Some(Event::WhisperReceived {
            sender: c.get("sender")?.to_owned(),
            text: c.get("text")?.to_owned(),
        });
    }
    let c = match_any(ENGLISH.whisper_sent, body)?;
    Some(Event::WhisperSent {
        recipient: c.get("recipient")?.to_owned(),
        text: c.get("text")?.to_owned(),
    })
}

/// Matches a system message, the body after its `: `, against the English locale table.
fn system_message(text: &str) -> Option<Event> {
    if let Some(c) = match_any(ENGLISH.zone_entered, text) {
        return Some(Event::ZoneEntered {
            area_name: c.get("area")?.to_owned(),
        });
    }
    if let Some(c) = match_any(ENGLISH.level_up, text) {
        return Some(Event::LevelUp {
            name: c.get("name")?.to_owned(),
            class: c.get("class")?.to_owned(),
            level: digits(c.get("level")?)?,
        });
    }
    if let Some(c) = match_any(ENGLISH.character_slain, text) {
        return Some(Event::CharacterSlain {
            name: c.get("name")?.to_owned(),
        });
    }
    if let Some(c) = match_any(ENGLISH.afk_on, text) {
        return Some(Event::AfkOn {
            autoreply: c.get("autoreply").map(str::to_owned),
        });
    }
    match_any(ENGLISH.afk_off, text).map(|_| Event::AfkOff)
}

fn area_generated(body: &str) -> Option<Event> {
    let captures = match_template(AREA_GENERATED, body)?;
    Some(Event::AreaGenerated {
        area_level: digits(captures.get("level")?)?,
        area_id: captures.get("area_id")?.to_owned(),
        seed: digits(captures.get("seed")?)?,
    })
}

fn scene_set(body: &str) -> Option<Event> {
    let captures = match_template(SCENE_SET, body)?;
    Some(Event::SceneSet {
        area_name: captures.get("area")?.to_owned(),
    })
}

/// Parses an all-ASCII-digit capture; a sign, other text or overflow gives `None`, never a panic.
fn digits<T: std::str::FromStr>(text: &str) -> Option<T> {
    if text.bytes().all(|b| b.is_ascii_digit()) {
        text.parse().ok()
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use crate::log::{Event, parse_line};
    fn ev(body: &str) -> Event {
        parse_line(&format!(
            "2026/01/02 03:04:05 123456789 1a2b3c4d [DEBUG Client 4242] {body}"
        ))
        .unwrap()
        .event
    }
    fn unknown(body: &str) -> bool {
        matches!(ev(body), Event::Unknown { .. })
    }
    fn sys(t: &str) -> Event {
        ev(&format!(": {t}"))
    }
    #[test]
    fn area_generated() {
        assert_eq!(
            ev(r#"Generating level 2 area "G1_2" with seed 3000000001"#),
            Event::AreaGenerated {
                area_level: 2,
                area_id: "G1_2".into(),
                seed: 3_000_000_001
            }
        );
        assert_eq!(
            ev(r#": Generating level 68 area "HideoutCanal" with seed 1"#),
            Event::AreaGenerated {
                area_level: 68,
                area_id: "HideoutCanal".into(),
                seed: 1
            }
        );
        assert_eq!(
            ev(r#"Generating level 79 area "c_g2_9_2_" with seed 18446744073709551615"#),
            Event::AreaGenerated {
                area_level: 79,
                area_id: "c_g2_9_2_".into(),
                seed: u64::MAX
            }
        );
        for b in [
            r#"Generating level x area "G1_2" with seed 1"#,
            r#"Generating level 2 area "G1_2" with seed 18446744073709551616"#,
            r#"Generating level 2 area "G1_2" with seed 5 extra"#,
            r#"Generating level 2 area G1_2 with seed 1"#,
            r#"Generating level 2 area "" with seed 1"#,
            r#"Generating level +2 area "G1_2" with seed 1"#,
            r#"$TraderJoe: Generating level 2 area "G1_2" with seed 1"#,
        ] {
            assert!(unknown(b), "{b}");
        }
    }
    #[test]
    fn scene_set_keeps_the_name_verbatim() {
        for n in [
            "Clearfell",
            "(null)",
            "(unknown)",
            "Mr. Smith's [Old] Hideout",
        ] {
            assert_eq!(
                ev(&format!("[SCENE] Set Source [{n}]")),
                Event::SceneSet {
                    area_name: n.into()
                }
            );
        }
        assert!(unknown("[SCENE] Set Source []") && unknown("[SCENE] Set Source [Clearfell] x"));
    }
    #[test]
    fn zone_entered_drops_one_final_period() {
        assert_eq!(
            sys("You have entered Clearfell."),
            Event::ZoneEntered {
                area_name: "Clearfell".into()
            }
        );
        assert_eq!(
            sys("You have entered Mr. Smith's Hideout."),
            Event::ZoneEntered {
                area_name: "Mr. Smith's Hideout".into()
            }
        );
        assert_eq!(
            sys("You have entered The Grelwood"),
            Event::ZoneEntered {
                area_name: "The Grelwood".into()
            }
        );
    }
    #[test]
    fn level_up_and_slain() {
        assert_eq!(
            sys("ExileOne (Smith of Kitava) is now level 12"),
            Event::LevelUp {
                name: "ExileOne".into(),
                class: "Smith of Kitava".into(),
                level: 12
            }
        );
        assert_eq!(
            sys("망명자 (Blood Mage) is now level 33"),
            Event::LevelUp {
                name: "망명자".into(),
                class: "Blood Mage".into(),
                level: 33
            }
        );
        assert_eq!(
            sys("ExileOne has been slain."),
            Event::CharacterSlain {
                name: "ExileOne".into()
            }
        );
    }
    #[test]
    fn afk_toggles() {
        assert_eq!(sys("AFK mode is now ON"), Event::AfkOn { autoreply: None });
        assert_eq!(sys("AFK mode is now ON."), Event::AfkOn { autoreply: None });
        assert_eq!(
            sys("AFK mode is now ON. Autoreply \"Back in 5\""),
            Event::AfkOn {
                autoreply: Some("Back in 5".into())
            }
        );
        assert_eq!(
            sys("AFK mode is now ON. Autoreply \"Back in 5\"."),
            Event::AfkOn {
                autoreply: Some("Back in 5".into())
            }
        );
        assert_eq!(sys("AFK mode is now OFF"), Event::AfkOff);
        assert_eq!(sys("AFK mode is now OFF."), Event::AfkOff);
    }
    #[test]
    fn whispers() {
        assert_eq!(
            ev("@From TraderJoe: Hi, is the bow still for sale?"),
            Event::WhisperReceived {
                sender: "TraderJoe".into(),
                text: "Hi, is the bow still for sale?".into()
            }
        );
        assert_eq!(
            ev("@To TraderJoe: see: the trade window"),
            Event::WhisperSent {
                recipient: "TraderJoe".into(),
                text: "see: the trade window".into()
            }
        );
        assert_eq!(
            ev("@From <GLD> TraderJoe: Hi"),
            Event::WhisperReceived {
                sender: "<GLD> TraderJoe".into(),
                text: "Hi".into()
            }
        );
        assert_eq!(
            ev("@From 망명자: 안녕하세요"),
            Event::WhisperReceived {
                sender: "망명자".into(),
                text: "안녕하세요".into()
            }
        );
        assert_eq!(
            ev("@From TraderJoe: : You have entered Clearfell."),
            Event::WhisperReceived {
                sender: "TraderJoe".into(),
                text: ": You have entered Clearfell.".into()
            }
        );
        for b in [
            "@From TraderJoe:",
            "@Fromage: hi",
            "#@From TraderJoe: hi",
            ": @From TraderJoe: hi",
            "@From : hi",
        ] {
            assert!(unknown(b), "{b}");
        }
    }
    #[test]
    fn look_alikes_and_unconfirmed_messages_are_unknown() {
        for t in [
            "ExileOne (Warrior) is now level 12.",
            "ExileOne (Warrior) is now level +5",
            "ExileOne (Warrior) is now level",
            "Exile One (Warrior) is now level 5",
            "ExileOne (Warrior) is now level 99999999999",
            "ExileOne has been slain",
            "ExileOne has committed suicide.",
            "AFK mode is now ONX",
            "AFK mode is now ON. Autoreply \"x\" extra",
            "AFK mode is now ON. Autoreply \"\"",
            "DND mode is now ON. Autoreply \"Busy\"",
            "DND mode is now OFF",
            "자리 비움 모드를 해제했습니다.",
        ] {
            assert!(matches!(sys(t), Event::Unknown { .. }), "{t}");
        }
        for b in [
            "$TraderJoe: : ExileOne (Warrior) is now level 12",
            "ExileOne: : ExileOne has been slain.",
            "%ExileTwo: x] : ExileOne (Warrior) is now level 5",
            ":You have entered Clearfell.",
            "You have entered Clearfell.",
        ] {
            assert!(unknown(b), "{b}");
        }
    }
}
