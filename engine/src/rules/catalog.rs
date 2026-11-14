//! Catalogue des règles pour l'interface : familles, exemples, valeurs permises. Les libellés français sont côté interface.

use serde_json::{Value, json};

use super::catalog_data::{ACTIONS, CONDITIONS, TRIGGERS};
use super::catalog_fields::fields_of;
use super::template::VARIABLES;

/// Transforme une ligne (famille, type, exemple) en objet JSON ; un exemple illisible donne `null`.
fn entry(family: &str, kind: &str, example: &str) -> Value {
    let example = serde_json::from_str::<Value>(example).unwrap_or(Value::Null);
    json!({"family": family, "type": kind, "fields": fields_of(kind, &example), "example": example})
}

/// Catalogue complet : déclencheurs, conditions, actions, variables de modèles et valeurs énumérées.
pub fn catalog() -> Value {
    json!({
        "triggers": TRIGGERS.iter().map(|(kind, example)| entry("", kind, example)).collect::<Vec<_>>(),
        "conditions": CONDITIONS.iter().map(|(family, kind, example)| entry(family, kind, example)).collect::<Vec<_>>(),
        "actions": ACTIONS.iter().map(|(family, kind, example)| entry(family, kind, example)).collect::<Vec<_>>(),
        "template_variables": VARIABLES,
        "match_ops": ["contains", "equals", "starts_with", "ends_with", "glob", "regex", "in_list"],
        "compare": ["lt", "le", "eq", "ne", "ge", "gt"],
        "units": ["seconds", "minutes", "hours", "days", "weeks", "months"],
        "weekdays": ["mon", "tue", "wed", "thu", "fri", "sat", "sun"],
        "kinds": ["image", "video", "audio", "document", "archive", "code", "other"],
        "conflict_policies": ["rename", "skip", "ask"],
        "limits": {"rules": 200, "alternatives": 10, "conditions": 30, "actions": 20, "value_length": 1000},
    })
}
