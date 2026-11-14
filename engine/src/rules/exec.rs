//! Exécution d'une action prévue : essai à blanc, famille d'exécuteur, résultat journalisable.

use super::exec_attrs;
use super::exec_common::action_name;
use super::exec_move;
use super::exec_names;
use super::exec_trash;
use super::outcome::PlannedAction;
use super::run_types::{ActionResult, Fail, Outcome, RunEnv, Step};

/// Cherche l'exécuteur de l'action ; une action qui relève de l'interface (notification, favoris, ouverture) est renvoyée à l'appelant.
fn dispatch(env: &RunEnv, planned: &PlannedAction, current: &mut String) -> Step {
    if let Some(step) = exec_names::run(env, planned, current) {
        return step;
    }
    if let Some(step) = exec_move::run(env, planned, current) {
        return step;
    }
    exec_trash::run(env, planned, current).or_else(|| exec_attrs::run(env, planned, current)).unwrap_or(Err(Fail::Defer))
}

/// Exécute une action sur le fichier courant ; une règle en essai ne fait que dire ce qu'elle aurait fait.
pub fn execute(env: &RunEnv, planned: &PlannedAction, current: &mut String) -> ActionResult {
    let mut result = ActionResult {
        rule_id: planned.rule_id.clone(),
        path: planned.path.clone(),
        action: action_name(&planned.action),
        outcome: Outcome::Done,
        detail: String::new(),
        undo: Vec::new(),
    };
    if planned.dry_run {
        (result.outcome, result.detail) = (Outcome::DryRun, planned.text.clone().unwrap_or_default());
        return result;
    }
    match dispatch(env, planned, current) {
        Ok(done) => (result.detail, result.undo) = (done.detail, done.undo),
        Err(Fail::Skip(reason)) => (result.outcome, result.detail) = (Outcome::Skipped, reason),
        Err(Fail::Error(message)) => (result.outcome, result.detail) = (Outcome::Failed, message),
        Err(Fail::Ask) => (result.outcome, result.detail) = (Outcome::Ask, "la destination existe déjà : choix demandé".into()),
        Err(Fail::Defer) => (result.outcome, result.detail) = (Outcome::Deferred, planned.text.clone().unwrap_or_default()),
    }
    result
}
