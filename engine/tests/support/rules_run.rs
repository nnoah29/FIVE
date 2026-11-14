//! Aides des tests d'exécution : planifier une règle sur un chemin et l'exécuter dans un bac à sable.

use five_engine::jobs::registry::Jobs;
use five_engine::rules::evaluate::{EventKind, evaluate};
use five_engine::rules::exec_pass::run_plan;
use five_engine::rules::outcome::PlannedAction;
use five_engine::rules::run_types::{ActionResult, RunEnv};
use five_engine::rules::types_rule::Rule;
use serde_json::{Value, json};

use super::Sandbox;
use super::rules_json::{file, rule_with};

/// Règle sur `Dl` (non en essai) pour tous les fichiers, avec les actions données.
pub fn any_file(actions: Value) -> Rule {
    rule_with(json!([{"test": {"type": "is_file"}}]), actions)
}

/// Plan d'une règle pour un chemin, sans exécuter.
pub fn plan(rule: &Rule, path: &str) -> Vec<PlannedAction> {
    evaluate(std::slice::from_ref(rule), &file(path), EventKind::Manual).actions
}

/// Plante et exécute une règle sur un chemin ; retourne les résultats.
pub fn run(sandbox: &Sandbox, rule: &Rule, path: &str) -> Vec<ActionResult> {
    let jobs = Jobs::new();
    let root = sandbox.root();
    run_plan(&RunEnv { root: &root, jobs: &jobs }, &plan(rule, path))
}
