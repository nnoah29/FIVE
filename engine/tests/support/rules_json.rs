//! Aides des tests de règles : construire une règle ou un contexte depuis du JSON, faux environnement de dossiers.

use five_engine::rules::context::FileContext;
use five_engine::rules::types_condition::Condition;
use five_engine::rules::types_rule::Rule;
use five_engine::rules::validate::{Env, FolderStatus};
use serde_json::{Value, json};

/// Règle complète depuis du JSON.
pub fn rule_from(value: Value) -> Rule {
    serde_json::from_value(value).unwrap()
}

/// Règle surveillant `Dl` (récursif) avec une alternative de conditions et des actions.
pub fn rule_with(conditions: Value, actions: Value) -> Rule {
    rule_from(json!({
        "id": "r1", "name": "Test", "enabled": true, "trial": false,
        "watch": [{"path": "Dl", "recursive": true}],
        "triggers": [{"type": "created"}],
        "alternatives": [{"conditions": conditions}],
        "actions": actions
    }))
}

/// Condition depuis du JSON `{negated?, test: {...}}`.
pub fn condition(value: Value) -> Condition {
    serde_json::from_value(value).unwrap()
}

/// Contexte d'un fichier de `Dl` avec une date courante fixe.
pub fn file(path: &str) -> FileContext {
    FileContext { now: 1_700_000_000.0, ..FileContext::new(path) }
}

/// Faux environnement : dossiers listés existent, `../` sort de la racine, `Corbeille` est la corbeille.
pub struct FakeEnv(pub Vec<&'static str>);

impl Env for FakeEnv {
    fn folder_status(&self, path: &str) -> FolderStatus {
        match path {
            p if p.starts_with("../") => FolderStatus::OutsideRoot,
            "Corbeille" => FolderStatus::InTrash,
            p if self.0.contains(&p) => FolderStatus::Ok,
            _ => FolderStatus::Missing,
        }
    }
}
