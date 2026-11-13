//! État Git des enfants directs d'un dossier : M modifié, A ajouté, D supprimé, ? non suivi, ! ignoré, U conflit.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Component, Path};

use crate::git_run::run_git;

const ORDER: &str = "UMADL?!";

/// Marque d'un code de statut `git status --porcelain` à deux lettres.
fn mark_for(code: &str) -> char {
    match code {
        "AA" | "DD" => 'U',
        "!!" => '!',
        "??" => '?',
        _ if code.contains('U') => 'U',
        _ if code.contains('D') => 'D',
        _ if code.contains('A') => 'A',
        _ => 'M',
    }
}

/// Garde la marque la plus importante selon l'ordre d'affichage.
fn merge_mark(old: Option<char>, new: char) -> char {
    match old {
        Some(old) if ORDER.find(old) <= ORDER.find(new) => old,
        _ => new,
    }
}

/// Nom du premier composant de `path` sous `folder`, si `path` est dedans.
fn first_child(folder: &Path, path: &Path) -> Option<String> {
    let rel = path.strip_prefix(folder).ok()?;
    match rel.components().next()? {
        Component::Normal(name) => Some(name.to_string_lossy().into_owned()),
        _ => None,
    }
}

/// Marques Git des enfants directs de `full` ; vide hors d'un dépôt ou en cas d'échec.
pub fn git_marks(full: &Path) -> BTreeMap<String, char> {
    let top = run_git(full, &["rev-parse", "--show-toplevel"]);
    let status = run_git(full, &["status", "--porcelain=v1", "-z", "--ignored=matching", "--untracked-files=normal"]);
    let (Some(top), Some(status)) = (top, status) else { return BTreeMap::new() };
    let top = fs::canonicalize(String::from_utf8_lossy(&top).trim()).unwrap_or_default();
    let mut marks: BTreeMap<String, char> = BTreeMap::new();
    for record in status.split(|byte| *byte == 0).filter(|r| r.len() >= 4) {
        let code = String::from_utf8_lossy(&record[..2]).into_owned();
        let path = top.join(String::from_utf8_lossy(&record[3..]).trim_end_matches('/'));
        if let Some(name) = first_child(full, &path) {
            let merged = merge_mark(marks.get(&name).copied(), mark_for(&code));
            marks.insert(name, merged);
        }
    }
    marks
}
