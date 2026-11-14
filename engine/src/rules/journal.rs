//! Journal des règles (`journal-regles.jsonl`) : une ligne par action, rotation à 5 Mo, aucun contenu de fichier.

use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use serde_json::{Value, json};

use super::run_types::ActionResult;
use crate::clock::local_stamp;

const MAX_BYTES: u64 = 5 * 1024 * 1024;

/// Emplacement du journal dans le dossier de configuration.
pub fn journal_path(dir: &Path) -> PathBuf {
    dir.join("journal-regles.jsonl")
}

/// Passe le journal en `.1` (l'ancien `.1` disparaît) quand il dépasse la taille maximale.
fn rotate(path: &Path) {
    if fs::metadata(path).is_ok_and(|m| m.len() > MAX_BYTES) {
        let _ = fs::rename(path, path.with_extension("jsonl.1"));
    }
}

/// Ajoute les résultats d'une passe (`run` les regroupe pour l'annulation).
pub fn append(path: &Path, run: &str, results: &[ActionResult]) -> io::Result<()> {
    rotate(path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    let stamp = local_stamp();
    for r in results {
        let line =
            json!({"t": stamp, "run": run, "rule": r.rule_id, "path": r.path, "action": r.action, "outcome": r.outcome, "detail": r.detail, "undo": r.undo});
        writeln!(file, "{line}")?;
    }
    Ok(())
}

/// Marque une passe comme annulée.
pub fn mark_undone(path: &Path, run: &str) -> io::Result<()> {
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    writeln!(file, "{}", json!({"t": local_stamp(), "undo_of": run}))
}

/// Toutes les lignes lisibles du journal, de la plus ancienne à la plus récente.
pub fn read_all(path: &Path) -> Vec<Value> {
    fs::read_to_string(path).unwrap_or_default().lines().filter_map(|line| serde_json::from_str(line).ok()).collect()
}

/// Les `limit` dernières lignes, de la plus récente à la plus ancienne.
pub fn read_recent(path: &Path, limit: usize) -> Vec<Value> {
    read_all(path).into_iter().rev().take(limit).collect()
}
