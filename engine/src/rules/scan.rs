//! Balayage des dossiers surveillés : fichiers et dossiers vus, dans un budget de temps.

use std::fs;
use std::time::{Instant, UNIX_EPOCH};

use super::evaluate::watch_contains;
use super::types_rule::Watch;
use crate::root::Root;

const SKIPPED_FOLDERS: [&str; 3] = [".git", "node_modules", "target"];

/// Un élément vu pendant le balayage.
#[derive(Debug, Clone, PartialEq)]
pub struct Seen {
    pub path: String,
    pub size: u64,
    pub mtime: f64,
    pub is_dir: bool,
}

/// Décrit une entrée du dossier ; `None` si elle a disparu.
fn describe(root: &Root, entry: &fs::DirEntry) -> Option<Seen> {
    let meta = entry.metadata().ok()?;
    let mtime = meta.modified().ok().and_then(|t| t.duration_since(UNIX_EPOCH).ok()).map_or(0.0, |d| d.as_secs_f64());
    Some(Seen { path: root.relative(&entry.path()), size: meta.len(), mtime, is_dir: meta.is_dir() && !meta.file_type().is_symlink() })
}

/// Indique si le balayage doit entrer dans ce dossier : récursif, profondeur permise, ni `.git`, ni `node_modules`, ni `target`.
fn descend(watch: &Watch, seen: &Seen) -> bool {
    let name = seen.path.rsplit('/').next().unwrap_or("");
    let base = watch.path.trim_matches('/');
    let rel = if base.is_empty() { seen.path.as_str() } else { seen.path.strip_prefix(&format!("{base}/")).unwrap_or("") };
    let depth = rel.matches('/').count() as u32;
    seen.is_dir
        && watch.recursive
        && !SKIPPED_FOLDERS.contains(&name)
        && watch.max_depth.is_none_or(|max| depth < max)
        && !(watch.ignore_hidden && name.starts_with('.'))
}

/// Balaye un dossier surveillé et ajoute ce qu'il contient à `out` ; retourne `false` si le budget de temps est dépassé.
pub fn scan_watch(root: &Root, watch: &Watch, deadline: Instant, out: &mut Vec<Seen>) -> bool {
    let Ok(start) = root.resolve(watch.path.trim_matches('/')) else { return true };
    let mut pending = vec![start];
    while let Some(folder) = pending.pop() {
        if Instant::now() > deadline {
            return false;
        }
        for entry in fs::read_dir(&folder).into_iter().flatten().flatten() {
            let Some(seen) = describe(root, &entry) else { continue };
            if descend(watch, &seen) {
                pending.push(entry.path());
            }
            if watch_contains(watch, &seen.path) {
                out.push(seen);
            }
        }
    }
    true
}
