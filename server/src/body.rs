//! Lecture des champs d'un corps JSON.

use serde_json::Value;

/// Texte d'un champ du corps JSON, vide s'il est absent ou n'est pas du texte.
pub fn text<'a>(body: &'a Value, key: &str) -> &'a str {
    body.get(key).and_then(Value::as_str).unwrap_or("")
}

/// Liste de textes d'un champ du corps JSON (les éléments non textuels sont ignorés).
pub fn texts(body: &Value, key: &str) -> Vec<String> {
    let list = body.get(key).and_then(Value::as_array);
    list.map_or_else(Vec::new, |items| items.iter().filter_map(|i| i.as_str().map(String::from)).collect())
}
