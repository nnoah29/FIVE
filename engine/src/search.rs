//! Recherche récursive par nom, simple ou floue, avec budget de temps.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use crate::entry::Entry;
use crate::fuzzy::fuzzy_score;
use crate::root::Root;

const BUDGET: Duration = Duration::from_secs(4);

/// Cherche `query` dans les noms sous `full` ; résultats triés par pertinence puis profondeur.
pub fn search_dir(root: &Root, full: &Path, query: &str, hidden: bool, fuzzy: bool) -> Vec<Entry> {
    let query = query.to_lowercase();
    let (limit, deadline) = (if fuzzy { 2000 } else { 400 }, Instant::now() + BUDGET);
    let mut found: Vec<(i64, Entry)> = Vec::new();
    let mut pending = vec![full.to_path_buf()];
    while let Some(folder) = pending.pop() {
        visit(root, &folder, (&query, hidden, fuzzy), &mut pending, &mut found);
        if found.len() >= limit || Instant::now() > deadline {
            break;
        }
    }
    found.sort_by_key(|(score, entry)| (*score, entry.path.len()));
    let items = found.into_iter().map(|(_, entry)| entry);
    if fuzzy { items.take(80).collect() } else { items.collect() }
}

/// Examine un dossier : ajoute les correspondances et empile les sous-dossiers.
fn visit(root: &Root, folder: &Path, how: (&str, bool, bool), pending: &mut Vec<PathBuf>, found: &mut Vec<(i64, Entry)>) {
    let (query, hidden, fuzzy) = how;
    let Ok(scan) = fs::read_dir(folder) else { return };
    for entry in scan.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if !hidden && name.starts_with('.') {
            continue;
        }
        if entry.file_type().is_ok_and(|kind| kind.is_dir()) {
            pending.push(entry.path());
        }
        let score = if fuzzy { fuzzy_score(query, &name) } else { name.to_lowercase().contains(query).then_some(0) };
        if let (Some(score), Some(item)) = (score, Entry::describe(root, &entry.path(), &name)) {
            found.push((score, item));
        }
    }
}
