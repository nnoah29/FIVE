//! Bac à sable de test : un faux dossier personnel jetable qui sert aussi de racine.
#![allow(dead_code)]

pub mod catalog_samples;
pub mod rules_helpers;
pub mod rules_json;
pub mod zip_bytes;

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};

use five_engine::root::Root;

static COUNTER: AtomicU32 = AtomicU32::new(0);

pub struct Sandbox {
    pub dir: PathBuf,
}

impl Sandbox {
    pub fn new() -> Self {
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!("five-test-{}-{id}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        Self { dir: fs::canonicalize(dir).unwrap() }
    }

    pub fn root(&self) -> Root {
        Root::new(&self.dir, &self.dir).unwrap()
    }

    pub fn write(&self, rel: &str, content: &str) -> PathBuf {
        let path = self.dir.join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, content).unwrap();
        path
    }
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.dir);
    }
}
