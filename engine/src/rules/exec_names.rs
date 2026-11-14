//! Exécution des actions de nom : renommages et masquage.

use super::exec_common::{leaf, target_free};
use super::outcome::PlannedAction;
use super::plan_text::is_rename;
use super::run_types::{Done, Fail, RunEnv, Step, Undo};
use super::types_action::RuleAction as A;
use crate::rename::rename_many;

/// Nom avec ou sans point initial.
fn hidden_name(name: &str, hide: bool) -> String {
    let bare = name.trim_start_matches('.');
    if hide { format!(".{bare}") } else { bare.to_string() }
}

/// Renomme `current` en `new_name` dans son dossier, sans jamais écraser ; met à jour le chemin courant.
fn rename_to(env: &RunEnv, planned: &PlannedAction, current: &mut String, new_name: &str) -> Step {
    let full = env.root.resolve_entry(current)?;
    let folder = full.parent().ok_or_else(|| Fail::Error("Dossier introuvable".into()))?;
    if new_name == leaf(&full) {
        return Err(Fail::Skip("le nom ne change pas".into()));
    }
    let target = target_free(folder, new_name, planned.on_conflict)?;
    rename_many(env.root, &[(current.clone(), leaf(&target))])?;
    *current = env.root.relative(&target);
    Ok(Done { detail: current.clone(), undo: vec![Undo::Move { from: full.to_string_lossy().into_owned(), to: target.to_string_lossy().into_owned() }] })
}

/// Exécute une action de renommage ou de masquage ; `None` pour les autres actions.
pub fn run(env: &RunEnv, planned: &PlannedAction, current: &mut String) -> Option<Step> {
    let name = current.rsplit('/').next().unwrap_or("").to_string();
    let new_name = match &planned.action {
        action if is_rename(action) => planned.text.clone()?,
        A::Hide { value } => hidden_name(&name, *value),
        _ => return None,
    };
    Some(rename_to(env, planned, current, &new_name))
}
