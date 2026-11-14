//! API d'écriture des règles : créer, modifier, supprimer, réordonner, activer.

use std::path::Path;

use serde_json::{Value, json};

use super::api_previews::{load_previews, save_previews};
use super::api_support::{parse_rule, require_valid, rule_hash, rules_file, view};
use super::store::{Stored, load, save_checked};
use super::validate::validate_set;
use crate::ids::unique_id;
use crate::refus::{Refus, Resultat};
use crate::root::Root;

/// Enregistre la liste après avoir validé l'ensemble (200 règles, identifiants uniques).
fn commit(dir: &Path, stored: &Stored) -> Resultat<()> {
    if let Some(problem) = validate_set(&stored.rules).first() {
        return Err(Refus::new(400, problem.message.clone()));
    }
    save_checked(&rules_file(dir), stored)
}

/// Crée une règle : toujours « en essai », identifiant attribué, validée avant d'être enregistrée.
pub fn create_rule(root: &Root, dir: &Path, body: &Value) -> Resultat<Value> {
    let mut rule = parse_rule(body)?;
    (rule.id, rule.trial) = (format!("r-{}", unique_id().chars().take(8).collect::<String>()), true);
    require_valid(root, &rule)?;
    let mut stored = load(&rules_file(dir));
    stored.rules.push(rule.clone());
    commit(dir, &stored)?;
    Ok(json!({"rule": view(root, &rule)}))
}

/// Modifie une règle : elle repasse « en essai » et son aperçu consulté est oublié.
pub fn update_rule(root: &Root, dir: &Path, id: &str, body: &Value) -> Resultat<Value> {
    let mut rule = parse_rule(body)?;
    (rule.id, rule.trial) = (id.to_string(), true);
    require_valid(root, &rule)?;
    let mut stored = load(&rules_file(dir));
    let slot = stored.rules.iter_mut().find(|r| r.id == id).ok_or_else(|| Refus::new(404, "Règle introuvable"))?;
    *slot = rule.clone();
    commit(dir, &stored)?;
    let mut previews = load_previews(dir);
    previews.remove(id);
    save_previews(dir, &previews)?;
    Ok(json!({"rule": view(root, &rule)}))
}

/// Supprime une règle.
pub fn delete_rule(dir: &Path, id: &str) -> Resultat<Value> {
    let mut stored = load(&rules_file(dir));
    let before = stored.rules.len();
    stored.rules.retain(|r| r.id != id);
    if stored.rules.len() == before {
        return Err(Refus::new(404, "Règle introuvable"));
    }
    commit(dir, &stored)?;
    let mut previews = load_previews(dir);
    previews.remove(id);
    save_previews(dir, &previews)?;
    Ok(json!({"ok": true}))
}

/// Change l'ordre d'évaluation : `ids` doit contenir exactement les règles existantes.
pub fn reorder_rules(dir: &Path, ids: &[String]) -> Resultat<Value> {
    let mut stored = load(&rules_file(dir));
    let mut sorted: Vec<&String> = ids.iter().collect();
    sorted.sort();
    let mut known: Vec<&String> = stored.rules.iter().map(|r| &r.id).collect();
    known.sort();
    if sorted != known {
        return Err(Refus::new(400, "La liste doit contenir exactement les règles existantes"));
    }
    stored.rules.sort_by_key(|r| ids.iter().position(|id| *id == r.id));
    commit(dir, &stored)?;
    Ok(json!({"ok": true}))
}

/// Active ou désactive une règle ; `activate` la sort de l'essai, seulement si son aperçu actuel a été consulté.
pub fn enable_rule(root: &Root, dir: &Path, id: &str, enabled: bool, activate: bool) -> Resultat<Value> {
    let mut stored = load(&rules_file(dir));
    let rule = stored.rules.iter_mut().find(|r| r.id == id).ok_or_else(|| Refus::new(404, "Règle introuvable"))?;
    if activate {
        require_valid(root, rule)?;
        if load_previews(dir).get(id) != Some(&rule_hash(rule)) {
            return Err(Refus::new(409, "Consultez d'abord l'aperçu de la règle avant de l'activer"));
        }
        rule.trial = false;
    }
    rule.enabled = enabled;
    let shown = view(root, rule);
    commit(dir, &stored)?;
    Ok(json!({"rule": shown}))
}
