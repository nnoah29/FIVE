//! Occupation disque : taille cumulée de chaque enfant direct d'un dossier, avec budget de temps.

use std::fs;
use std::io;
use std::path::Path;
use std::time::{Duration, Instant};

use serde_json::{Value, json};

use crate::root::Root;

const MAX_ROWS: usize = 200;

/// Taille cumulée des fichiers sous `path` ; le booléen indique que le temps a manqué.
fn dir_size(path: &Path, deadline: Instant) -> (u64, bool) {
    let (mut size, mut pending) = (0, vec![path.to_path_buf()]);
    while let Some(folder) = pending.pop() {
        if Instant::now() > deadline {
            return (size, true);
        }
        for entry in fs::read_dir(&folder).into_iter().flatten().flatten() {
            match entry.file_type() {
                Ok(kind) if kind.is_dir() => pending.push(entry.path()),
                _ => size += entry.metadata().map_or(0, |m| m.len()),
            }
        }
    }
    (size, false)
}

/// Taille de chaque enfant direct de `full`, du plus gros au plus petit, et total.
pub fn usage_of(root: &Root, full: &Path, budget: Duration) -> io::Result<Value> {
    let deadline = Instant::now() + budget;
    let mut children: Vec<_> = fs::read_dir(full)?.flatten().collect();
    children.sort_by_key(|e| e.file_name());
    let (mut rows, mut partial) = (Vec::new(), false);
    for entry in children {
        let is_dir = entry.file_type().is_ok_and(|kind| kind.is_dir());
        let (size, late) = if is_dir { dir_size(&entry.path(), deadline) } else { (entry.metadata().map_or(0, |m| m.len()), false) };
        partial |= late;
        rows.push((entry.file_name().to_string_lossy().into_owned(), is_dir, size, root.relative(&entry.path())));
    }
    rows.sort_by_key(|row| std::cmp::Reverse(row.2));
    let total: u64 = rows.iter().map(|row| row.2).sum();
    let items: Vec<Value> = rows.iter().take(MAX_ROWS).map(|(n, d, s, p)| json!({"n": n, "d": d, "s": s, "p": p})).collect();
    Ok(json!({"items": items, "total": total, "partial": partial || Instant::now() > deadline}))
}
