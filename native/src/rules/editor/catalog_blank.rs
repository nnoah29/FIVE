//! Valeur par défaut d'un champ d'un élément neuf.

use serde_json::{Map, Value};

/// Valeur d'un champ d'un élément neuf : on garde les choix du catalogue, les modèles et les heures ; le reste est vidé.
pub fn blank_field(key: &str, field: &Value, value: &Value) -> Value {
    let keep_text = field["choices"].is_array() || key == "at" || key == "format" || value.as_str().is_some_and(|t| t.contains('{'));
    match value {
        Value::String(_) if !keep_text => Value::String(String::new()),
        Value::Array(items) if items.iter().all(Value::is_string) && field["choices"].is_null() => Value::Array(Vec::new()),
        Value::Object(map) if key == "matcher" => {
            Value::Object(map.iter().map(|(k, v)| (k.clone(), if k == "value" { Value::String(String::new()) } else { v.clone() })).collect::<Map<_, _>>())
        }
        Value::Array(_) if key == "tests" => Value::Array(Vec::new()),
        other => other.clone(),
    }
}
