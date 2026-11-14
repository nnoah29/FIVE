//! Aides partagées des tests de règles : règle sur les PDF, textes calculés, codes de validation.

use five_engine::rules::context::FileContext;
use five_engine::rules::evaluate::{EventKind, evaluate};
use five_engine::rules::types_rule::Rule;
use five_engine::rules::validate::validate_rule;
use serde_json::{Value, json};

use super::rules_json::{FakeEnv, file, rule_with};

/// Règle sur les fichiers `.pdf` de `Dl`, avec l'identifiant donné.
pub fn pdf_rule(id: &str, actions: Value) -> Rule {
    let mut rule = good(actions);
    rule.id = id.into();
    rule
}

/// Règle valable sur les `.pdf` (conditions fixes) avec les actions données.
pub fn good(actions: Value) -> Rule {
    rule_with(json!([{"test": {"type": "extension", "in_list": ["pdf"]}}]), actions)
}

/// Textes calculés par les actions pour un fichier, dans l'ordre.
pub fn texts(ctx: &FileContext, actions: Value) -> Vec<Option<String>> {
    let rule = rule_with(json!([{"test": {"type": "is_file"}}]), actions);
    evaluate(&[rule], ctx, EventKind::Created).actions.into_iter().map(|a| a.text).collect()
}

/// Fichier daté du 15 janvier 2024, compteur à 4.
pub fn dated() -> FileContext {
    FileContext { modified: Some(1_705_320_000.0), counter: 4, ..file("Dl/Mon Été.JPEG") }
}

/// Codes des problèmes d'une règle, avec des dossiers `Dl`, `Docs` et `Docs/Photos` existants.
pub fn codes(rule: &Rule) -> Vec<&'static str> {
    validate_rule(rule, &FakeEnv(vec!["Dl", "Docs", "Docs/Photos"])).into_iter().map(|p| p.code).collect()
}
