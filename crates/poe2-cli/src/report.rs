//! Discovery output for `paths` and `doctor` (ADRs 0012, 0015 and 0016).

use std::fmt::Write;
use std::path::PathBuf;

use poe2_core::locate::{Discovery, Found};

/// Each kind's name and candidates, in Discovery order.
pub(crate) fn kinds(d: &Discovery) -> [(&'static str, &[Found<PathBuf>]); 5] {
    [
        ("installs", &d.installs),
        ("client_log", &d.client_log),
        ("user_dir", &d.user_dir),
        ("build_planner", &d.build_planner),
        ("filters", &d.filters),
    ]
}

/// `<kind>\t<path>\n` for the first existing candidate of each kind.
pub(crate) fn paths_text(d: &Discovery) -> String {
    let mut text = String::new();
    for (kind, found) in kinds(d) {
        if let Some(f) = found.iter().find(|f| f.exists) {
            let _ = writeln!(text, "{kind}\t{}", f.value.display()); // writing to a String cannot fail
        }
    }
    text
}

/// Whether any candidate of any kind exists.
pub(crate) fn any_exists(d: &Discovery) -> bool {
    kinds(d)
        .iter()
        .any(|(_, found)| found.iter().any(|f| f.exists))
}

/// One padded row per candidate, a count of those that exist, and a hint when no client log exists.
pub(crate) fn doctor_table(d: &Discovery) -> String {
    let mut rows = vec![["KIND", "SOURCE", "EXISTS", "PATH", "NOTE"].map(String::from)];
    for (kind, found) in kinds(d) {
        rows.extend(found.iter().map(|f| {
            [
                kind.to_owned(),
                format!("{:?}", f.source),
                if f.exists { "yes" } else { "no" }.to_owned(),
                f.value.display().to_string(),
                f.note.clone().unwrap_or_default(),
            ]
        }));
    }
    let mut widths = [0; 5];
    for row in &rows {
        for (w, cell) in widths.iter_mut().zip(row) {
            *w = (*w).max(cell.chars().count());
        }
    }
    let mut text = String::new();
    for row in &rows {
        let mut line = String::new();
        for (w, cell) in widths.iter().zip(row) {
            let _ = write!(line, "{cell:<w$}  "); // writing to a String cannot fail
        }
        text.push_str(line.trim_end());
        text.push('\n');
    }
    let all = rows.len().saturating_sub(1); // less the header
    let existing = kinds(d)
        .iter()
        .flat_map(|(_, found)| found.iter())
        .filter(|f| f.exists)
        .count();
    let _ = writeln!(text, "{existing} of {all} locations exist.");
    if !d.client_log.iter().any(|f| f.exists) {
        text.push_str(
            "No client log exists. Set POE2_INSTALL_DIR to the game folder, or pass --log to poe2 tail.\n",
        );
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;
    use poe2_core::locate::{Discovery, Found, Source};

    pub(crate) fn f(p: &str, s: Source, e: bool) -> Found<std::path::PathBuf> {
        Found {
            value: p.into(),
            source: s,
            exists: e,
            note: None,
        }
    }

    #[test]
    fn paths_text_lists_the_first_existing_candidate_of_each_kind() {
        let d = Discovery {
            installs: vec![
                f("/env", Source::EnvOverride, false),
                f("/a", Source::Steam, true),
                f("/b", Source::Steam, true),
            ],
            client_log: vec![f("/a/logs/Client.txt", Source::Steam, true)],
            ..Default::default()
        };
        assert_eq!(
            paths_text(&d),
            "installs\t/a\nclient_log\t/a/logs/Client.txt\n"
        );
        assert!(any_exists(&d) && !any_exists(&Discovery::default()));
    }

    #[test]
    fn doctor_table_is_exact() {
        let d = Discovery {
            installs: vec![Found {
                note: Some("POE2_INSTALL_DIR".into()),
                ..f("/games/poe2", Source::EnvOverride, true)
            }],
            client_log: vec![f("/games/poe2/logs/Client.txt", Source::EnvOverride, false)],
            ..Default::default()
        };
        assert_eq!(
            doctor_table(&d),
            "\
KIND        SOURCE       EXISTS  PATH                         NOTE
installs    EnvOverride  yes     /games/poe2                  POE2_INSTALL_DIR
client_log  EnvOverride  no      /games/poe2/logs/Client.txt
1 of 2 locations exist.
No client log exists. Set POE2_INSTALL_DIR to the game folder, or pass --log to poe2 tail.
"
        );
    }
}
