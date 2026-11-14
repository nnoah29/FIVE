//! Règles automatiques : valider, prévisualiser, appliquer, annuler, activer, réordonner.

use five_engine::refus::{Refus, Resultat};
use five_engine::rules::api_run::{preview_rule, run_rule, undo_rules, validate_body};
use five_engine::rules::api_write::{enable_rule, reorder_rules};
use serde_json::Value;

use crate::api_rules::config_dir;
use crate::body::{text, texts};
use crate::reply::{Reply, json_reply};
use crate::state::State;

/// `POST /api/rules/<action>` : `validate`, `preview`, `run`, `undo`, `enable` ou `reorder`.
pub fn rules_action(state: &State, action: &str, body: &Value) -> Resultat<Reply> {
    let (root, jobs, dir) = (&state.root, &state.jobs, config_dir(state));
    let answer = match action {
        "validate" => validate_body(root, body)?,
        "preview" => preview_rule(root, jobs, &dir, body)?,
        "run" => run_rule(root, jobs, &dir, body)?,
        "undo" => undo_rules(root, &dir)?,
        "enable" => enable_rule(root, &dir, text(body, "id"), body["enabled"].as_bool().unwrap_or(true), body["activate"].as_bool().unwrap_or(false))?,
        "reorder" => reorder_rules(&dir, &texts(body, "ids"))?,
        _ => return Err(Refus::new(404, "Route inconnue")),
    };
    Ok(json_reply(200, &answer))
}
