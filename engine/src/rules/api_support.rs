//! Aides de l'API des règles : lecture d'une règle, empreinte, validation, vue JSON, aperçus consultés.

use std::path::{Path, PathBuf};

use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use super::status::{RuleStatus, rule_status};
use super::types_rule::Rule;
use super::validate::{RootEnv, validate_rule};
use crate::refus::{Refus, Resultat};
use crate::root::Root;

/// Fichier des règles dans le dossier de configuration.
pub fn rules_file(dir: &Path) -> PathBuf {
    dir.join("regles.json")
}

/// Lit une règle envoyée par l'interface (les champs absents sont vides, la validation les signale) ; un format illisible est un refus lisible.
pub fn parse_rule(value: &Value) -> Resultat<Rule> {
    let mut filled = value.clone();
    let object = filled.as_object_mut().ok_or_else(|| Refus::new(400, "Règle illisible : un objet JSON est attendu"))?;
    for (key, empty) in
        [("id", json!("")), ("name", json!("")), ("watch", json!([])), ("triggers", json!([])), ("alternatives", json!([])), ("actions", json!([]))]
    {
        object.entry(key).or_insert(empty);
    }
    serde_json::from_value(filled).map_err(|e| Refus::new(400, format!("Règle illisible : {e}")))
}

/// Empreinte de la règle : sert à savoir si l'aperçu consulté correspond à la règle actuelle.
pub fn rule_hash(rule: &Rule) -> String {
    let text = serde_json::to_string(rule).unwrap_or_default();
    Sha256::digest(text.as_bytes()).iter().map(|b| format!("{b:02x}")).collect()
}

/// Refuse une règle invalide en listant tous les problèmes dans le message.
pub fn require_valid(root: &Root, rule: &Rule) -> Resultat<()> {
    let problems = validate_rule(rule, &RootEnv(root));
    if problems.is_empty() { Ok(()) } else { Err(Refus::new(400, problems.iter().map(|p| p.message.clone()).collect::<Vec<_>>().join(" "))) }
}

/// Règle et son statut calculé, tels que l'interface les lit.
pub fn view(root: &Root, rule: &Rule) -> Value {
    let status = rule_status(rule, &RootEnv(root));
    let reason = if let RuleStatus::Invalid(reason) = &status { reason.clone() } else { String::new() };
    let mut value = serde_json::to_value(rule).unwrap_or(Value::Null);
    value["status"] = json!(status.as_str());
    value["reason"] = json!(reason);
    value
}
