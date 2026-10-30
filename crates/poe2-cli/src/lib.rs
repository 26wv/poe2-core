//! The `poe2` command (ADR 0016): an internal library for in-process tests, not a stable API.
#![deny(unsafe_code)]

use std::ffi::OsString;
use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

use clap::{Parser, Subcommand};
use poe2_core::locate::Discovery;
use poe2_core::log::LogTail;

mod format;
mod report;

#[derive(Parser)]
#[command(
    name = "poe2",
    version,
    about = "Find Path of Exile 2 files and read its client log"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Report every location searched and whether it exists.
    Doctor {
        /// Print the Discovery as JSON.
        #[arg(long)]
        json: bool,
    },
    /// Print the first existing path of each kind.
    Paths {
        /// Print the Discovery as JSON.
        #[arg(long)]
        json: bool,
    },
    /// Follow the client log and print new events.
    Tail {
        /// The log file to follow instead of the discovered client log.
        #[arg(long, value_name = "PATH")]
        log: Option<PathBuf>,
        /// Print the existing lines first.
        #[arg(long)]
        from_start: bool,
        /// Print one JSON object per log line.
        #[arg(long)]
        json: bool,
    },
    /// Print the events in a log file.
    Parse {
        /// The log file to read.
        file: PathBuf,
        /// Print one JSON object per log line.
        #[arg(long)]
        json: bool,
    },
}

/// Runs `poe2` with `args` (the program name first) and returns the exit code.
pub fn run<I, T>(args: I, out: &mut dyn Write, err: &mut dyn Write) -> u8
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let cli = match Cli::try_parse_from(args) {
        Ok(cli) => cli,
        Err(e) => {
            // A closed stream is not an error (ADR 0016).
            let _ = if e.use_stderr() {
                write!(err, "{}", e.render())
            } else {
                write!(out, "{}", e.render())
            };
            return u8::try_from(e.exit_code()).unwrap_or(2);
        }
    };
    match cli.command {
        Command::Doctor { json } => doctor(json, out, err),
        Command::Paths { json } => paths(json, out, err),
        Command::Tail {
            log,
            from_start,
            json,
        } => tail(log, from_start, json, out, err),
        Command::Parse { file, json } => parse(&file, json, out, err),
    }
}

/// `poe2 parse FILE`: streams every log line of FILE to `out` (ADR 0016).
fn parse(file: &Path, json: bool, out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    let reader = match File::open(file) {
        Ok(f) => f,
        Err(e) => return cannot_read(err, file, e),
    };
    let mut out = BufWriter::new(out);
    for line in poe2_core::log::read_lines(reader) {
        let written = match line {
            Ok(line) => format::write_line(&mut out, &line, json),
            Err(e) => return cannot_read(err, file, e),
        };
        if let Err(e) = written {
            return write_failed(err, e);
        }
    }
    match out.flush() {
        Ok(()) => 0,
        Err(e) => write_failed(err, e),
    }
}

/// `poe2 tail`: follows `log` or the first existing client log, polling every 250 ms until killed
/// (ADRs 0014 and 0016); 1 when there is no log to follow.
fn tail(
    log: Option<PathBuf>,
    from_start: bool,
    json: bool,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> u8 {
    let found = log.or_else(|| {
        let logs = poe2_core::locate::discover().client_log;
        logs.into_iter().find(|f| f.exists).map(|f| f.value)
    });
    let Some(path) = found else {
        let _ = writeln!(
            err,
            "poe2: no client log found; run `poe2 doctor` to see what was checked, or pass --log PATH"
        );
        return 1;
    };
    let opened = if from_start {
        LogTail::open_from_start(&path)
    } else {
        LogTail::open(&path)
    };
    let mut follower = match opened {
        Ok(t) => t,
        Err(e) => return cannot_read(err, &path, e),
    };
    let _ = writeln!(err, "following {}", path.display());
    let mut out = BufWriter::new(out);
    loop {
        let written = match follower.next_line() {
            Ok(Some(line)) => format::write_line(&mut out, &line, json),
            // Caught up: show what was written, then wait for the next poll.
            Ok(None) => out
                .flush()
                .map(|()| thread::sleep(Duration::from_millis(250))),
            Err(e) => return cannot_read(err, &path, e),
        };
        if let Err(e) = written {
            return write_failed(err, e);
        }
    }
}

/// `poe2 doctor`: every candidate as a table, or the Discovery JSON; 0 even when nothing exists.
fn doctor(json: bool, out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    let d = poe2_core::locate::discover();
    match write_discovery(&d, json, report::doctor_table, out) {
        Ok(()) => 0,
        Err(e) => write_failed(err, e),
    }
}

/// `poe2 paths`: the first existing path of each kind, or the Discovery JSON; 1 when nothing exists.
fn paths(json: bool, out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    let d = poe2_core::locate::discover();
    match write_discovery(&d, json, report::paths_text, out) {
        Err(e) => write_failed(err, e),
        Ok(()) if report::any_exists(&d) => 0,
        Ok(()) => 1,
    }
}

/// Writes `d` as pretty JSON, or as `text(d)`, and flushes.
fn write_discovery(
    d: &Discovery,
    json: bool,
    text: fn(&Discovery) -> String,
    out: &mut dyn Write,
) -> io::Result<()> {
    let text = if json {
        serde_json::to_string_pretty(d).map(|s| s + "\n")?
    } else {
        text(d)
    };
    out.write_all(text.as_bytes())?;
    out.flush()
}

/// Reports a file that can't be opened or read; exits 2 (ADR 0016).
fn cannot_read(err: &mut dyn Write, path: &Path, e: io::Error) -> u8 {
    let _ = writeln!(err, "poe2: cannot read {}: {e}", path.display());
    2
}

/// A closed stdout ends the command quietly with 0; any other write error exits 2 (ADR 0016).
fn write_failed(err: &mut dyn Write, e: io::Error) -> u8 {
    if e.kind() == io::ErrorKind::BrokenPipe {
        return 0;
    }
    let _ = writeln!(err, "poe2: cannot write output: {e}");
    2
}
