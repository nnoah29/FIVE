//! Lecture d'un aperçu texte, avec détection de contenu binaire.

use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

use serde_json::{Value, json};

const TEXT_LIMIT: usize = 200_000;
const SNIFF: usize = 8192;

/// Lit au plus 200 Ko d'un fichier ; un octet nul au début le déclare binaire.
pub fn read_text(full: &Path) -> io::Result<Value> {
    let mut chunk = Vec::new();
    File::open(full)?.take(TEXT_LIMIT as u64 + 1).read_to_end(&mut chunk)?;
    if chunk.iter().take(SNIFF).any(|byte| *byte == 0) {
        return Ok(json!({"binary": true, "text": "", "truncated": false}));
    }
    let truncated = chunk.len() > TEXT_LIMIT;
    chunk.truncate(TEXT_LIMIT);
    Ok(json!({"binary": false, "text": String::from_utf8_lossy(&chunk), "truncated": truncated}))
}
