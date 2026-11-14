//! Annulation de la dernière passe : chaque action réversible est défaite, dans l'ordre inverse.

use std::collections::HashSet;
use std::fs::{self, File};
use std::path::Path;
use std::time::{Duration, UNIX_EPOCH};

use serde_json::Value;

use super::journal::{mark_undone, read_all};
use super::run_types::Undo;
use crate::root::Root;
use crate::trash_ops::move_to_trash;

/// Bilan d'une annulation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UndoReport {
    pub run: String,
    pub undone: usize,
    pub failed: Vec<String>,
}

/// Indique si le chemin est dans la racine ou dans la corbeille : le journal ne peut pas faire agir ailleurs.
fn allowed(root: &Root, path: &Path) -> bool {
    root.inside(path) || root.in_trash(path)
}

/// Défait une action ; ne touche jamais à ce qui a changé depuis (destination absente ou origine occupée : échec lisible).
fn undo_one(root: &Root, undo: &Undo) -> Result<(), String> {
    let check = |path: &str| if allowed(root, Path::new(path)) { Ok(()) } else { Err("chemin hors de la racine".to_string()) };
    match undo {
        Undo::Move { from, to } | Undo::Trashed { original: from, trashed: to } => {
            check(from).and(check(to))?;
            if !Path::new(to).exists() || Path::new(from).exists() {
                return Err("l'élément a déjà changé : annulation refusée".into());
            }
            fs::rename(to, from).map_err(|e| e.to_string())?;
            if matches!(undo, Undo::Trashed { .. }) {
                let name = Path::new(to).file_name().map_or_else(String::new, |n| n.to_string_lossy().into_owned());
                let _ = fs::remove_file(root.trash_dir().join("info").join(format!("{name}.trashinfo")));
            }
            Ok(())
        }
        Undo::TrashCreated { path } => {
            check(path)?;
            if Path::new(path).exists() { move_to_trash(root, Path::new(path)).map_err(|r| r.message) } else { Ok(()) }
        }
        Undo::RemoveEmptyDir { path } => check(path).and_then(|_| fs::remove_dir(path).or(Ok(())).map_err(|e: std::io::Error| e.to_string())),
        Undo::Recreate { path } => check(path).and_then(|_| fs::create_dir_all(path).map_err(|e| e.to_string())),
        Undo::Mode { path, mode } => {
            check(path).and_then(|_| fs::set_permissions(path, std::os::unix::fs::PermissionsExt::from_mode(*mode & 0o777)).map_err(|e| e.to_string()))
        }
        Undo::Time { path, seconds } => check(path)
            .and_then(|_| File::open(path).and_then(|f| f.set_modified(UNIX_EPOCH + Duration::from_secs_f64(seconds.max(0.0)))).map_err(|e| e.to_string())),
    }
}

/// Dernière passe qui a quelque chose à défaire et n'a pas déjà été annulée.
fn last_undoable(lines: &[Value]) -> Option<String> {
    let undone: HashSet<&str> = lines.iter().filter_map(|l| l["undo_of"].as_str()).collect();
    let undoable = |l: &&Value| l["outcome"] == "done" && l["undo"].as_array().is_some_and(|u| !u.is_empty());
    lines.iter().rev().filter(undoable).filter_map(|l| l["run"].as_str()).find(|run| !undone.contains(run)).map(String::from)
}

/// Annule la dernière passe annulable du journal ; `None` s'il n'y en a pas.
pub fn undo_last(root: &Root, journal: &Path) -> Option<UndoReport> {
    let lines = read_all(journal);
    let run = last_undoable(&lines)?;
    let undos: Vec<Undo> = lines
        .iter()
        .filter(|l| l["run"] == run.as_str())
        .flat_map(|l| l["undo"].as_array().cloned().unwrap_or_default())
        .filter_map(|u| serde_json::from_value(u).ok())
        .collect();
    let mut report = UndoReport { run: run.clone(), undone: 0, failed: Vec::new() };
    for undo in undos.iter().rev() {
        match undo_one(root, undo) {
            Ok(()) => report.undone += 1,
            Err(reason) => report.failed.push(reason),
        }
    }
    let _ = mark_undone(journal, &run);
    Some(report)
}
