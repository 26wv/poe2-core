//! Peak heap of `poe2 parse` on a large file with an over-long line (ADR 0014; ADR 0016).
// copied from crates/poe2-core/tests/bounded_memory.rs
use std::alloc::{GlobalAlloc, Layout, System};
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

const LINE: &[u8] = b"2026/01/02 03:04:05 123456789 1a2b3c4d [INFO Client 4242] : ExileOne (Smith of Kitava) is now level 12\r\n";

#[test]
fn parse_streams_with_bounded_memory() {
    use std::io::Write;
    let path = std::env::temp_dir().join(format!("poe2-cli-memory-{}.txt", std::process::id()));
    {
        let mut f = std::io::BufWriter::new(std::fs::File::create(&path).unwrap());
        for _ in 0..200_000 {
            f.write_all(LINE).unwrap();
        }
        f.write_all(&vec![b'x'; 8 << 20]).unwrap();
        f.write_all(b"\n").unwrap();
    }
    let peak = peak_during(|| {
        assert_eq!(
            poe2_cli::run(
                ["poe2", "parse", path.to_str().unwrap(), "--json"],
                &mut std::io::sink(),
                &mut std::io::sink()
            ),
            0
        )
    });
    std::fs::remove_file(&path).unwrap();
    assert!(peak < 4 << 20, "poe2 parse peaked at {peak} bytes");
}
