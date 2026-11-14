//! Exécution des déplacements, copies et archives, par la file de transferts.

use std::fs;

use super::exec_archive::{unzip_one, zip_one};
use super::exec_common::{leaf, run_job, safe_relative, target_free};
use super::outcome::PlannedAction;
use super::run_types::{Done, Fail, RunEnv, Step, Undo};
use super::types_action::RuleAction as A;

/// Déplace ou copie `current` vers le dossier prévu (créé au besoin), sans écraser ; `move` met à jour le chemin courant.
fn transfer(env: &RunEnv, planned: &PlannedAction, current: &mut String, kind: &str) -> Step {
    let dest = safe_relative(planned.text.as_deref().unwrap_or(""))?;
    let (src, dest_full) = (env.root.resolve_entry(current)?, env.root.path().join(&dest));
    fs::create_dir_all(&dest_full)?;
    env.root.resolve(&dest)?;
    let target = target_free(&dest_full, &leaf(&src), planned.on_conflict)?;
    if target == src {
        return Err(Fail::Skip("déjà à cet endroit".into()));
    }
    run_job(env, kind, std::slice::from_ref(current), &dest)?;
    let (from, to) = (src.to_string_lossy().into_owned(), target.to_string_lossy().into_owned());
    *current = if kind == "move" { env.root.relative(&target) } else { current.clone() };
    Ok(Done { detail: env.root.relative(&target), undo: vec![if kind == "move" { Undo::Move { from, to } } else { Undo::TrashCreated { path: to } }] })
}

/// Exécute un déplacement, une copie ou une action d'archive ; `None` pour les autres actions.
pub fn run(env: &RunEnv, planned: &PlannedAction, current: &mut String) -> Option<Step> {
    Some(match &planned.action {
        A::MoveTo { .. } | A::MoveToDated { .. } | A::MoveToByKind { .. } | A::MoveToByExtension { .. } => transfer(env, planned, current, "move"),
        A::CopyTo { .. } => transfer(env, planned, current, "copy"),
        A::Zip { mode: super::types_action::ZipMode::OnePerItem, .. } => zip_one(env, planned, current, false),
        A::ZipFolderThenTrash => zip_one(env, planned, current, true),
        A::Unzip { then } => unzip_one(env, current, *then),
        _ => return None,
    })
}
