use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};

static COUNTER: AtomicU32 = AtomicU32::new(0);

pub fn sample() -> PathBuf {
    let id = COUNTER.fetch_add(1, Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!("five-chooser-{}-{id}", std::process::id()));
    fs::create_dir_all(dir.join("alpha")).unwrap();
    fs::create_dir_all(dir.join("Beta")).unwrap();
    for name in ["a.txt", "b.png", "C.md", ".hidden"] {
        fs::write(dir.join(name), "x").unwrap();
    }
    dir
}
