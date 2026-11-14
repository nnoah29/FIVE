//! Description des champs d'une condition, d'une action ou d'un déclencheur, déduite de son exemple : l'interface en tire un formulaire.

use serde_json::{Value, json};

const COMPARE: [&str; 6] = ["lt", "le", "eq", "ne", "ge", "gt"];
const WEEKDAYS: [&str; 7] = ["mon", "tue", "wed", "thu", "fri", "sat", "sun"];
const KINDS: [&str; 7] = ["image", "video", "audio", "document", "archive", "code", "other"];
const GIT: [&str; 6] = ["modified", "added", "deleted", "untracked", "ignored", "conflict"];
const DATES: [&str; 4] = ["modified", "created", "today", "name"];

/// Valeurs permises pour un champ énuméré, selon le type de l'élément et le nom du champ ; `None` pour un champ libre.
fn choices_for(kind: &str, key: &str) -> Option<Vec<&'static str>> {
    Some(match (kind, key) {
        (_, "cmp") => COMPARE.to_vec(),
        (_, "days") => WEEKDAYS.to_vec(),
        ("kind", "in_list") => KINDS.to_vec(),
        ("git_status", "in_list") => GIT.to_vec(),
        ("change_case", "mode") => vec!["lower", "upper", "title", "sentence"],
        ("zip", "mode") => vec!["one_per_item", "one_for_all"],
        (_, "from") => DATES.to_vec(),
        (_, "position") => vec!["prefix", "suffix"],
        (_, "keep") => vec!["newest", "oldest", "shortest_name"],
        (_, "then") => vec!["keep", "trash_archive"],
        (_, "scope") => vec!["folder", "watched", "root"],
        ("free_space_below", "on") => vec!["folder", "disk"],
        ("orientation", "value") => vec!["landscape", "portrait", "square"],
        ("encoding", "value") => vec!["utf8", "other"],
        ("weekly", "day") => WEEKDAYS.to_vec(),
        _ => return None,
    })
}

/// Genre d'un champ : composé (comparaison de texte, durée, conditions), date, taille, droits, dossier, liste, nombre, case ou texte.
fn field_kind(kind: &str, key: &str, value: &Value) -> &'static str {
    match (key, value) {
        ("matcher", _) => "matcher",
        ("duration" | "for" | "every", _) => "duration",
        ("tests", _) => "conditions",
        ("at", Value::Number(_)) => "date",
        ("bytes", _) => "bytes",
        ("mode", Value::Number(_)) if kind == "permissions" || kind == "set_permissions" => "octal",
        ("folder" | "base" | "path", Value::String(_)) if kind != "daily" => "folder",
        (_, Value::Bool(_)) => "bool",
        (_, Value::Number(_)) => "number",
        (_, Value::Array(items)) if items.first().is_some_and(Value::is_number) || (items.is_empty() && matches!(key, "months" | "years")) => "numbers",
        (_, Value::Array(_)) => "list",
        _ => "text",
    }
}

/// Champs d'un élément du catalogue : nom, genre, valeurs permises ; le nom d'archive facultatif de `zip` est ajouté.
pub fn fields_of(kind: &str, example: &Value) -> Vec<Value> {
    let mut fields: Vec<Value> = example
        .as_object()
        .into_iter()
        .flatten()
        .filter(|(key, _)| *key != "type")
        .map(|(key, value)| {
            let mut field = json!({"name": key, "kind": field_kind(kind, key, value)});
            if let Some(choices) = choices_for(kind, key) {
                field["choices"] = json!(choices);
            }
            field
        })
        .collect();
    if kind == "zip" {
        fields.push(json!({"name": "name_template", "kind": "text", "optional": true}));
    }
    fields
}
