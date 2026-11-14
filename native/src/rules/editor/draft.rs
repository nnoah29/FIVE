//! Brouillon d'une règle : lecture de la règle enregistrée, nettoyage avant envoi.

use five_engine::rules::api_read::list_rules;
use serde_json::Value;

use crate::app::shared::App;
use crate::model::rules_draft::new_rule_draft;

/// Brouillon à éditer : la règle enregistrée (sans statut calculé) ou une règle neuve.
pub fn load_draft(app: &App, id: &Option<String>) -> Value {
    let Some(id) = id else { return new_rule_draft() };
    let list = list_rules(&app.root, &app.rules_dir());
    let mut rule = list["rules"].as_array().and_then(|rules| rules.iter().find(|r| r["id"] == id.as_str())).cloned().unwrap_or_else(new_rule_draft);
    if let Some(object) = rule.as_object_mut() {
        object.remove("status");
        object.remove("reason");
    }
    rule
}

/// Brouillon prêt à envoyer : un nom d'archive vide et une profondeur vide disparaissent.
pub fn cleaned(draft: &Value) -> Value {
    let mut out = draft.clone();
    for action in out["actions"].as_array_mut().into_iter().flatten() {
        if action["name_template"] == "" {
            action.as_object_mut().map(|o| o.remove("name_template"));
        }
    }
    for watch in out["watch"].as_array_mut().into_iter().flatten() {
        if watch["max_depth"].is_null() {
            watch.as_object_mut().map(|o| o.remove("max_depth"));
        }
    }
    out
}
