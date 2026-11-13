//! Emplacements de l'utilisateur : corbeille et dossiers XDG présents sous la racine.

use std::fs;
use std::path::Path;

use serde_json::{Value, json};

use crate::realpath::real_path;
use crate::root::Root;

const PLACES: [(&str, &str, &str, &str); 6] = [
    ("desktop", "Bureau", "DESKTOP", "Desktop"),
    ("downloads", "Téléchargements", "DOWNLOAD", "Downloads"),
    ("docs", "Documents", "DOCUMENTS", "Documents"),
    ("pictures", "Images", "PICTURES", "Pictures"),
    ("music", "Musique", "MUSIC", "Music"),
    ("videos", "Vidéos", "VIDEOS", "Videos"),
];

/// Chemin de la corbeille relatif au dossier personnel.
pub const TRASH: &str = ".local/share/Trash/files";

/// Dossiers définis dans `user-dirs.dirs` : clé (`DOWNLOAD`…) vers chemin absolu.
pub fn xdg_user_dirs(home: &Path) -> Vec<(String, String)> {
    let text = fs::read_to_string(home.join(".config/user-dirs.dirs")).unwrap_or_default();
    let home_text = home.to_string_lossy();
    text.lines()
        .filter_map(|line| {
            let (key, value) = line.split_once('=')?;
            let key = key.strip_prefix("XDG_")?.strip_suffix("_DIR")?;
            let value = value.trim().trim_matches('"').replace("$HOME", &home_text);
            Some((key.to_string(), value))
        })
        .collect()
}

/// Emplacements existants : corbeille, puis dossiers XDG sous la racine (hors racine elle-même).
pub fn existing_places(root: &Root) -> Vec<Value> {
    let mut out = Vec::new();
    let mut taken: Vec<String> = Vec::new();
    if root.path().join(TRASH).is_dir() {
        out.push(json!({"id": "trash", "label": "Corbeille", "path": TRASH}));
        taken.push(TRASH.to_string());
    }
    let xdg = xdg_user_dirs(root.home());
    for (id, label, key, default) in PLACES {
        let configured = xdg.iter().find(|(k, _)| k == key).map(|(_, v)| v.clone());
        let full = real_path(&configured.map_or_else(|| root.path().join(default), Into::into));
        let rel = root.relative(&full);
        if root.inside(&full) && full.is_dir() && full != root.path() && !taken.contains(&rel) {
            out.push(json!({"id": id, "label": label, "path": rel}));
            taken.push(rel);
        }
    }
    out
}
