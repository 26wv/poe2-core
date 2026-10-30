//! Peak heap of `read_lines` and `LogTail` on large input (docs/research/client-log.md "File properties"; ADR 0014).
// One #[test] fn, because the counters are process-wide.

use std::alloc::{GlobalAlloc, Layout, System};
use std::io::{self, Read, Write};
use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};

struct Counting;
static NOW: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        let p = unsafe { System.alloc(l) };
        if !p.is_null() {
            let n = NOW.fetch_add(l.size(), Relaxed) + l.size();
            PEAK.fetch_max(n, Relaxed);
        }
        p
    }
    unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
        unsafe { System.dealloc(p, l) };
        NOW.fetch_sub(l.size(), Relaxed);
    }
}
#[global_allocator]
static ALLOC: Counting = Counting;

fn peak_during(f: impl FnOnce()) -> usize {
    let base = NOW.load(Relaxed);
    PEAK.store(base, Relaxed);
    f();
    PEAK.load(Relaxed) - base
}

const LIMIT: usize = 2 << 20;
const LINE: &[u8] = b"2026/01/02 03:04:05 123456789 1a2b3c4d [INFO Client 4242] : ExileOne (Smith of Kitava) is now level 12\r\n";

/// `n` bytes of LINE repeated.
struct Repeat {
    pos: usize,
    left: usize,
}
impl Read for Repeat {
    fn read(&mut self, b: &mut [u8]) -> io::Result<usize> {
        let n = b.len().min(self.left);
        for x in &mut b[..n] {
            *x = LINE[self.pos % LINE.len()];
            self.pos += 1;
        }
        self.left -= n;
        Ok(n)
    }
}

#[test]
fn memory_stays_bounded() {
    let mb = 1 << 20;
    let input = Repeat {
        pos: 0,
        left: 28 * mb,
    }
    .chain(io::repeat(b'x').take(8 * mb as u64))
    .chain(&b"\n"[..])
    .chain(Repeat {
        pos: 0,
        left: 28 * mb,
    });
    let peak = peak_during(|| {
        assert!(
            poe2_core::log::read_lines(input)
                .filter(|l| l.is_ok())
                .count()
                > 500_000
        )
    });
    assert!(peak < LIMIT, "read_lines peaked at {peak} bytes");
    let path = std::env::temp_dir().join(format!("poe2-core-bounded-{}.txt", std::process::id()));
    let mut file = std::fs::File::create(&path).unwrap();
    io::copy(&mut io::repeat(b'x').take(16 * mb as u64), &mut file).unwrap();
    file.write_all(b"\n").unwrap();
    for _ in 0..50_000 {
        file.write_all(LINE).unwrap();
    }
    drop(file);
    let peak = peak_during(|| {
        let mut tail = poe2_core::log::LogTail::open_from_start(&path).unwrap();
        let mut n = 0;
        while tail.next_line().unwrap().is_some() {
            n += 1;
        }
        assert_eq!(n, 50_001);
    });
    std::fs::remove_file(&path).unwrap();
    assert!(peak < LIMIT, "LogTail peaked at {peak} bytes");
}
