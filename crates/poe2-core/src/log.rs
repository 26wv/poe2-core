//! Log parser: turns `Client.txt` lines into typed events (docs/research/client-log.md; ADRs 0007, 0008, 0012).

mod event;
mod line;
mod locale;
mod read;
mod split;
mod tail;

pub use event::Event;
pub use line::{LogLine, Prefix, Timestamp, parse_line};
pub use read::{ReadLines, read_lines};
pub use tail::LogTail;
