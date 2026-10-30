//! The frozen public types stay `Send + Sync`, which the bindings rely on (ADR 0022).

use std::fs::File;
use std::path::PathBuf;

use poe2_core::locate::{Discovery, Found, Source};
use poe2_core::log::{Event, LogLine, LogTail, Prefix, ReadLines, Timestamp};

fn assert_send_sync<T: Send + Sync>() {}

#[test]
fn public_types_are_send_and_sync() {
    assert_send_sync::<Discovery>();
    assert_send_sync::<Found<PathBuf>>();
    assert_send_sync::<Source>();
    assert_send_sync::<LogLine>();
    assert_send_sync::<Prefix>();
    assert_send_sync::<Timestamp>();
    assert_send_sync::<Event>();
    assert_send_sync::<LogTail>();
    assert_send_sync::<ReadLines<File>>();
}
