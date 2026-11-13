//! Un élément du système de fichiers tel que l'interface le voit.

use std::fs;
use std::path::Path;
use std::time::UNIX_EPOCH;

use serde_json::{Value, json};

use crate::mime;
use crate::root::Root;

/// Description d'un fichier ou dossier, avec son chemin relatif à la racine.
#[derive(Debug, Clone)]
pub struct Entry {
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
    pub modified: f64,
    pub is_link: bool,
    pub mime: Option<String>,
    pub path: String,
    pub children: Option<Option<u64>>,
}

impl Entry {
    /// Décrit un élément ; `None` s'il a disparu entre-temps.
    pub fn describe(root: &Root, full: &Path, name: &str) -> Option<Entry> {
        let info = fs::symlink_metadata(full).ok()?;
        let is_link = info.file_type().is_symlink();
        let target = if is_link { fs::metadata(full).unwrap_or(info) } else { info };
        let is_dir = fs::metadata(full).is_ok_and(|meta| meta.is_dir());
        let modified = target.modified().ok().and_then(|t| t.duration_since(UNIX_EPOCH).ok());
        Some(Entry {
            name: name.to_string(),
            is_dir,
            size: if is_dir { 0 } else { target.len() },
            modified: modified.map_or(0.0, |d| d.as_secs_f64()),
            is_link,
            mime: (!is_dir).then(|| mime::guess(name).to_string()),
            path: root.relative(full),
            children: None,
        })
    }

    /// Représentation JSON attendue par l'interface.
    pub fn to_json(&self) -> Value {
        let mut value = json!({"n": self.name, "d": self.is_dir, "s": self.size, "m": self.modified,
            "l": self.is_link, "t": self.mime, "p": self.path});
        if let Some(count) = self.children {
            value["c"] = json!(count);
        }
        value
    }
}
