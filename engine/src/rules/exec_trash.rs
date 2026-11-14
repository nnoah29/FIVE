//! Exécution des actions de nettoyage : corbeille, vidage de la corbeille, dossiers vides, aplatissement.

use std::fs;

use super::exec_common::{leaf, target_free};
use super::outcome::PlannedAction;
use super::run_types::{Done, Fail, RunEnv, Step, Undo};
use super::time_calc::seconds;
use super::types_action::RuleAction as A;
use super::types_matcher::DurationSpec;
use super::types_rule::ConflictPolicy;
use crate::trash_ops::{erase_for_good, move_to_trash_at};

/// Met l'élément à la corbeille XDG ; refuse un dossier qui contient la corbeille.
fn trash_item(env: &RunEnv, current: &str) -> Step {
    let full = env.root.resolve_entry(current)?;
    if env.root.protects_trash(&full) {
        return Err(Fail::Error("Ce dossier est protégé : il contient la corbeille".into()));
    }
    if env.root.in_trash(&full) {
        return Err(Fail::Skip("déjà dans la corbeille".into()));
    }
    let trashed = move_to_trash_at(env.root, &full)?;
    Ok(Done {
        detail: "corbeille".into(),
        undo: vec![Undo::Trashed { original: full.to_string_lossy().into_owned(), trashed: trashed.to_string_lossy().into_owned() }],
    })
}

/// Efface définitivement un élément de la corbeille plus vieux que la durée ; seul effacement définitif permis.
fn empty_trash(env: &RunEnv, current: &str, older: &DurationSpec) -> Step {
    let full = env.root.resolve_entry(current)?;
    if !env.root.in_trash(&full) {
        return Err(Fail::Skip("hors de la corbeille : effacement définitif refusé".into()));
    }
    let age = fs::symlink_metadata(&full)?.modified()?.elapsed().unwrap_or_default().as_secs_f64();
    if age < seconds(older) {
        return Err(Fail::Skip("pas assez ancien".into()));
    }
    erase_for_good(env.root, &full)?;
    Ok(Done { detail: "effacé définitivement".into(), undo: Vec::new() })
}

/// Supprime un dossier vide (jamais un dossier qui contient quelque chose).
fn remove_empty(env: &RunEnv, current: &str) -> Step {
    let full = env.root.resolve_entry(current)?;
    if !full.is_dir() || fs::read_dir(&full)?.next().is_some() {
        return Err(Fail::Skip("ce n'est pas un dossier vide".into()));
    }
    fs::remove_dir(&full)?;
    Ok(Done { detail: "dossier vide supprimé".into(), undo: vec![Undo::Recreate { path: full.to_string_lossy().into_owned() }] })
}

/// Remonte le contenu d'un dossier qui ne contient qu'un dossier ; s'arrête sans rien faire si un nom est déjà pris.
fn flatten(env: &RunEnv, current: &str, policy: ConflictPolicy) -> Step {
    let full = env.root.resolve_entry(current)?;
    let mut children = fs::read_dir(&full)?.flatten();
    let (Some(only), None) = (children.next(), children.next()) else { return Err(Fail::Skip("le dossier n'a pas un seul enfant".into())) };
    if !only.path().is_dir() || only.file_type()?.is_symlink() {
        return Err(Fail::Skip("l'unique enfant n'est pas un dossier".into()));
    }
    let inner: Vec<_> = fs::read_dir(only.path())?.flatten().collect();
    let moves: Vec<_> = inner
        .iter()
        .map(|e| {
            target_free(&full, &leaf(&e.path()), if policy == ConflictPolicy::Ask { ConflictPolicy::Ask } else { ConflictPolicy::Skip }).map(|t| (e.path(), t))
        })
        .collect::<Result<_, _>>()?;
    for (from, to) in &moves {
        fs::rename(from, to)?;
    }
    fs::remove_dir(only.path())?;
    Ok(Done { detail: format!("{} élément(s) remontés", moves.len()), undo: Vec::new() })
}

/// Exécute une action de nettoyage ; `None` pour les autres actions.
pub fn run(env: &RunEnv, planned: &PlannedAction, current: &str) -> Option<Step> {
    Some(match &planned.action {
        A::Trash | A::TrashDuplicates { .. } => trash_item(env, current),
        A::EmptyTrashOlderThan { duration } => empty_trash(env, current, duration),
        A::RemoveEmptyFolders => remove_empty(env, current),
        A::FlattenFolder => flatten(env, current, planned.on_conflict),
        _ => return None,
    })
}
