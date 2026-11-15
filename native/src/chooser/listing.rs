//! Contenu d'un dossier pour le sélecteur : dossiers d'abord, puis fichiers, triés par nom.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use super::matching::filter_accepts;
use super::request::{Filter, Mode};

/// Une ligne de la liste.
#[derive(Debug, Clone, PartialEq)]
pub struct Row {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub size: u64,
    pub modified: f64,
}

/// Décrit une entrée de dossier ; `None` si elle a disparu entre-temps.
fn describe(entry: &fs::DirEntry) -> Option<Row> {
    let meta = fs::metadata(entry.path()).or_else(|_| entry.metadata()).ok()?;
    let modified = meta.modified().ok().and_then(|t| t.duration_since(UNIX_EPOCH).ok()).map_or(0.0, |d| d.as_secs_f64());
    Some(Row { name: entry.file_name().to_string_lossy().into_owned(), path: entry.path(), is_dir: meta.is_dir(), size: meta.len(), modified })
}

/// Lit un dossier ; les éléments cachés ne sont gardés que si `hidden` est vrai.
pub fn read_rows(dir: &Path, hidden: bool) -> io::Result<Vec<Row>> {
    let mut rows: Vec<Row> = fs::read_dir(dir)?.flatten().filter_map(|e| describe(&e)).collect();
    rows.retain(|r| hidden || !r.name.starts_with('.'));
    rows.sort_by(|a, b| b.is_dir.cmp(&a.is_dir).then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase())));
    Ok(rows)
}

/// Garde ce que le mode permet de choisir : seulement des dossiers pour « Dossier », les fichiers filtrés sinon.
pub fn selectable(rows: Vec<Row>, mode: Mode, filter: Option<&Filter>) -> Vec<Row> {
    let keep = |row: &Row| match mode {
        Mode::Folder => row.is_dir,
        Mode::Open | Mode::Save => row.is_dir || filter.is_none_or(|f| filter_accepts(f, &row.name)),
    };
    rows.into_iter().filter(keep).collect()
}
