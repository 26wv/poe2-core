//! Find Path of Exile 2's files on disk and parse its `Client.txt` log into typed events.
//!
//! A passive file reader: no network, no game memory, no input automation.
#![deny(unsafe_code)]
#![warn(missing_docs)]

pub mod locate;
pub mod log;

#[cfg(test)]
mod test_support;
