//! Ligne du sélecteur convertie en élément du moteur, pour partager la fiche de la fenêtre principale.

use super::listing::Row;
use five_engine::entry::Entry;

/// Description du moteur pour une ligne : ce qu'il faut à l'aperçu partagé avec la fenêtre principale.
pub fn entry_of(row: &Row) -> Entry {
    Entry {
        name: row.name.clone(),
        is_dir: row.is_dir,
        size: row.size,
        modified: row.modified,
        is_link: false,
        mime: (!row.is_dir).then(|| five_engine::mime::guess(&row.name).to_string()),
        path: row.path.to_string_lossy().into_owned(),
        children: None,
    }
}
