//! API d'exécution des règles : valider, prévisualiser, appliquer maintenant, annuler.

use std::path::Path;
use std::sync::Arc;

use serde_json::{Value, json};

use super::api_previews::{load_previews, save_previews};
use super::api_support::{parse_rule, require_valid, rule_hash, rules_file, view};
use super::journal::journal_path;
use super::outcome::PlannedAction;
use super::runner::RulesService;
use super::status::{RuleStatus, rule_status};
use super::store::load;
use super::types_rule::Rule;
use super::undo::undo_last;
use super::validate::{RootEnv, validate_rule};
use crate::clock::now_seconds;
use crate::jobs::registry::Jobs;
use crate::refus::{Refus, Resultat};
use crate::root::Root;

const MAX_ITEMS: usize = 500;

/// Valide une règle sans l'enregistrer : problèmes (code, message) et statut qu'elle aurait.
pub fn validate_body(root: &Root, body: &Value) -> Resultat<Value> {
    let rule = parse_rule(body)?;
    let problems: Vec<Value> = validate_rule(&rule, &RootEnv(root)).into_iter().map(|p| json!({"code": p.code, "message": p.message})).collect();
    Ok(json!({"valid": problems.is_empty(), "problems": problems, "rule": view(root, &rule)}))
}

/// Règle visée par une requête : enregistrée (`id`) ou envoyée telle quelle (`rule`).
fn target_rule(dir: &Path, body: &Value) -> Resultat<Rule> {
    match (body["id"].as_str(), body.get("rule")) {
        (Some(id), _) => load(&rules_file(dir)).rules.into_iter().find(|r| r.id == id).ok_or_else(|| Refus::new(404, "Règle introuvable")),
        (None, Some(rule)) => parse_rule(rule),
        _ => Err(Refus::new(400, "Indiquez « id » ou « rule »")),
    }
}

/// Liste d'actions prévues pour l'interface.
fn planned_json(planned: &[PlannedAction]) -> Vec<Value> {
    planned
        .iter()
        .take(MAX_ITEMS)
        .map(|p| json!({"path": p.path, "action": super::exec_common::action_name(&p.action), "text": p.text, "dry_run": p.dry_run}))
        .collect()
}

/// Aperçu : ce que la règle ferait (500 fichiers au plus), sans rien modifier ; l'aperçu d'une règle enregistrée est retenu pour l'activer.
pub fn preview_rule(root: &Root, jobs: &Arc<Jobs>, dir: &Path, body: &Value) -> Resultat<Value> {
    let rule = target_rule(dir, body)?;
    require_valid(root, &rule)?;
    let paths: Option<Vec<String>> = body["paths"].as_array().map(|a| a.iter().filter_map(|p| p.as_str().map(String::from)).collect());
    let planned = RulesService::new(root.clone(), jobs.clone(), dir.to_path_buf()).preview(&rule, paths.as_deref(), now_seconds());
    if body["id"].is_string() && paths.is_none() {
        let mut previews = load_previews(dir);
        previews.insert(rule.id.clone(), rule_hash(&rule));
        save_previews(dir, &previews)?;
    }
    Ok(json!({"count": planned.len(), "truncated": planned.len() > MAX_ITEMS, "items": planned_json(&planned)}))
}

/// Applique maintenant une règle enregistrée, sur des fichiers précis ou sur tout son dossier surveillé.
pub fn run_rule(root: &Root, jobs: &Arc<Jobs>, dir: &Path, body: &Value) -> Resultat<Value> {
    let rule = target_rule(dir, &json!({"id": body["id"]}))?;
    match rule_status(&rule, &RootEnv(root)) {
        RuleStatus::Invalid(reason) => return Err(Refus::new(400, reason)),
        RuleStatus::Disabled => return Err(Refus::new(409, "La règle est désactivée")),
        _ => {}
    }
    let paths: Option<Vec<String>> = body["paths"].as_array().map(|a| a.iter().filter_map(|p| p.as_str().map(String::from)).collect());
    let report = RulesService::new(root.clone(), jobs.clone(), dir.to_path_buf()).run_now(Some(&rule.id), paths.as_deref(), now_seconds());
    let results: Vec<Value> =
        report.results.iter().take(MAX_ITEMS).map(|r| json!({"path": r.path, "action": r.action, "outcome": r.outcome, "detail": r.detail})).collect();
    Ok(json!({"run": report.run, "count": report.results.len(), "results": results}))
}

/// Annule la dernière exécution annulable ; 404 s'il n'y en a pas.
pub fn undo_rules(root: &Root, dir: &Path) -> Resultat<Value> {
    let report = undo_last(root, &journal_path(dir)).ok_or_else(|| Refus::new(404, "Rien à annuler"))?;
    Ok(json!({"run": report.run, "undone": report.undone, "failed": report.failed}))
}
