//! Contenu d'un dossier : éléments décrits, avec le nombre d'enfants des sous-dossiers.

use std::fs;
use std::io;
use std::path::Path;

use crate::entry::Entry;
use crate::root::Root;

const COUNTED_FOLDERS: usize = 300;

/// Nombre d'éléments d'un dossier, `None` s'il est illisible.
pub fn count_children(full: &Path, hidden: bool) -> Option<u64> {
    let scan = fs::read_dir(full).ok()?;
    let visible = scan.flatten().filter(|e| hidden || !e.file_name().to_string_lossy().starts_with('.'));
    Some(visible.count() as u64)
}

/// Liste un dossier ; les éléments cachés ne sont inclus que si `hidden`.
pub fn list_dir(root: &Root, full: &Path, hidden: bool) -> io::Result<Vec<Entry>> {
    let mut items = Vec::new();
    for found in fs::read_dir(full)? {
        let name = found?.file_name().to_string_lossy().into_owned();
        if !hidden && name.starts_with('.') {
            continue;
        }
        items.extend(Entry::describe(root, &full.join(&name), &name));
    }
    for item in items.iter_mut().filter(|i| i.is_dir).take(COUNTED_FOLDERS) {
        item.children = Some(count_children(&root.path().join(&item.path), hidden));
    }
    Ok(items)
}
