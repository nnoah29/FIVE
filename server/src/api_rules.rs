//! Règles automatiques : lecture (liste, catalogue, journal), création, modification, suppression.

use std::collections::HashMap;
use std::path::PathBuf;

use five_engine::refus::Resultat;
use five_engine::rules::api_read::{list_rules, rules_catalog, rules_history};
use five_engine::rules::api_write::{create_rule, delete_rule, update_rule};
use serde_json::Value;

use crate::reply::{Reply, json_reply};
use crate::state::State;

/// Dossier de configuration de FIVE : `~/.config/five`.
pub fn config_dir(state: &State) -> PathBuf {
    state.root.home().join(".config/five")
}

/// `GET /api/rules` : règles et statuts.
pub fn rules_list(state: &State) -> Reply {
    json_reply(200, &list_rules(&state.root, &config_dir(state)))
}

/// `GET /api/rules/catalog` : catalogue des conditions, actions et déclencheurs.
pub fn rules_catalog_reply() -> Reply {
    json_reply(200, &rules_catalog())
}

/// `GET /api/rules/history?limit=` : dernières lignes du journal.
pub fn rules_history_reply(state: &State, query: &HashMap<String, String>) -> Reply {
    let limit = query.get("limit").and_then(|l| l.parse().ok()).unwrap_or(50);
    json_reply(200, &rules_history(&config_dir(state), limit))
}

/// `POST /api/rules` : crée une règle (toujours en essai).
pub fn rule_create(state: &State, body: &Value) -> Resultat<Reply> {
    Ok(json_reply(201, &create_rule(&state.root, &config_dir(state), body)?))
}

/// `PUT /api/rules/<id>` : modifie une règle ; `DELETE /api/rules/<id>` la supprime si `body` est `None`.
pub fn rule_change(state: &State, id: &str, body: Option<&Value>) -> Resultat<Reply> {
    let dir = config_dir(state);
    match body {
        Some(body) => Ok(json_reply(200, &update_rule(&state.root, &dir, id, body)?)),
        None => Ok(json_reply(200, &delete_rule(&dir, id)?)),
    }
}
