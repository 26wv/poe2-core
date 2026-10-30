//! The locale table and its template matcher (docs/research/client-log.md Conflicting 5–6; ADRs 0007, 0009).

/// One language's templates for the system messages and whisper markers (ADR 0009), each list tried in order.
pub(crate) struct Locale {
    pub(crate) zone_entered: &'static [&'static str],
    pub(crate) level_up: &'static [&'static str],
    pub(crate) character_slain: &'static [&'static str],
    pub(crate) afk_on: &'static [&'static str],
    pub(crate) afk_off: &'static [&'static str],
    pub(crate) whisper_received: &'static [&'static str],
    pub(crate) whisper_sent: &'static [&'static str],
}

/// The English locale table, the only one v1 ships (ADR 0009).
pub(crate) const ENGLISH: Locale = Locale {
    zone_entered: &["You have entered {area}.", "You have entered {area}"],
    level_up: &["{name} ({class}) is now level {level}"],
    character_slain: &["{name} has been slain."],
    afk_on: &[
        "AFK mode is now ON. Autoreply \"{autoreply}\".",
        "AFK mode is now ON. Autoreply \"{autoreply}\"",
        "AFK mode is now ON.",
        "AFK mode is now ON",
    ],
    afk_off: &["AFK mode is now OFF.", "AFK mode is now OFF"],
    whisper_received: &["@From {sender}: {text}"],
    whisper_sent: &["@To {recipient}: {text}"],
};

/// The named values a template captured, in template order.
pub(crate) struct Captures<'a>(Vec<(&'static str, &'a str)>);

impl<'a> Captures<'a> {
    /// The value captured by the placeholder `{name}`, if the template has one.
    pub(crate) fn get(&self, name: &str) -> Option<&'a str> {
        self.0
            .iter()
            .find(|(key, _)| *key == name)
            .map(|(_, value)| *value)
    }
}

/// Matches `text` against one template, or returns `None` when it doesn't fit.
///
/// A template is literal text with `{name}` placeholders, and two placeholders always have
/// literal text between them. Literals must match exactly. A placeholder before the final
/// literal captures up to that suffix; before any other literal, up to its first occurrence; at
/// the end, the rest of the text. The whole text must be consumed. Captures are non-empty,
/// except `{text}`, and a `{name}` capture has no space (ADR 0007 "Matching rules").
pub(crate) fn match_template<'a>(template: &'static str, text: &'a str) -> Option<Captures<'a>> {
    let mut parts = template.split('{').peekable();
    let mut rest = text.strip_prefix(parts.next()?)?;
    let mut captures = Vec::new();
    while let Some(part) = parts.next() {
        let (name, literal) = part.split_once('}')?;
        let value = if literal.is_empty() {
            std::mem::take(&mut rest)
        } else if parts.peek().is_none() {
            let value = rest.strip_suffix(literal)?;
            rest = "";
            value
        } else {
            let (value, after) = rest.split_once(literal)?;
            rest = after;
            value
        };
        if (value.is_empty() && name != "text") || (name == "name" && value.contains(' ')) {
            return None;
        }
        captures.push((name, value));
    }
    rest.is_empty().then_some(Captures(captures))
}

/// Matches `text` against each template in order and returns the first match.
pub(crate) fn match_any<'a>(templates: &[&'static str], text: &'a str) -> Option<Captures<'a>> {
    templates
        .iter()
        .find_map(|template| match_template(template, text))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn caps(t: &'static str, s: &str) -> Option<Vec<(String, String)>> {
        match_template(t, s).map(|c| {
            c.0.iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect()
        })
    }
    fn kv(pairs: &[(&str, &str)]) -> Option<Vec<(String, String)>> {
        Some(
            pairs
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
        )
    }
    #[test]
    fn level_up_shape() {
        let t = "{name} ({class}) is now level {level}";
        assert_eq!(
            caps(t, "ExileOne (Smith of Kitava) is now level 12"),
            kv(&[
                ("name", "ExileOne"),
                ("class", "Smith of Kitava"),
                ("level", "12")
            ])
        );
        assert_eq!(
            caps(t, "망명자 (Warrior) is now level 5"),
            kv(&[("name", "망명자"), ("class", "Warrior"), ("level", "5")])
        );
        assert_eq!(caps(t, "Exile One (Warrior) is now level 5"), None);
        assert_eq!(caps(t, "ExileOne () is now level 5"), None);
    }
    #[test]
    fn a_final_literal_anchors_at_the_end() {
        assert_eq!(
            caps(
                "You have entered {area}.",
                "You have entered Mr. Smith's Hideout."
            ),
            kv(&[("area", "Mr. Smith's Hideout")])
        );
        assert_eq!(
            caps(
                "[SCENE] Set Source [{area}]",
                "[SCENE] Set Source [Mr. Smith's [Old] Hideout]"
            ),
            kv(&[("area", "Mr. Smith's [Old] Hideout")])
        );
        assert_eq!(
            caps("[SCENE] Set Source [{area}]", "[SCENE] Set Source []"),
            None
        );
        assert_eq!(caps("AFK mode is now ON", "AFK mode is now ONX"), None);
    }
    #[test]
    fn whispers_split_at_the_first_colon_space() {
        let t = "@From {sender}: {text}";
        assert_eq!(
            caps(t, "@From <GLD> TraderJoe: price: 5 ex"),
            kv(&[("sender", "<GLD> TraderJoe"), ("text", "price: 5 ex")])
        );
        assert_eq!(
            caps(t, "@From TraderJoe: "),
            kv(&[("sender", "TraderJoe"), ("text", "")])
        );
        assert_eq!(caps(t, "@From : hi"), None);
        assert_eq!(caps(t, "@Fro"), None);
    }
    #[test]
    fn match_any_takes_the_first_matching_template() {
        let t = &[
            "AFK mode is now ON. Autoreply \"{autoreply}\".",
            "AFK mode is now ON. Autoreply \"{autoreply}\"",
            "AFK mode is now ON.",
            "AFK mode is now ON",
        ];
        assert_eq!(
            match_any(t, "AFK mode is now ON. Autoreply \"Back in 5\".")
                .and_then(|c| c.get("autoreply")),
            Some("Back in 5")
        );
        assert_eq!(
            match_any(t, "AFK mode is now ON. Autoreply \"x\"").and_then(|c| c.get("autoreply")),
            Some("x")
        );
        assert!(match_any(t, "AFK mode is now ON").is_some_and(|c| c.get("autoreply").is_none()));
    }
}
