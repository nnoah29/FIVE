//! Résumé d'une règle en une ligne pour la liste : dossiers, déclencheurs, nombre de conditions et d'actions.

use serde_json::Value;

use super::rules_labels::trigger_label;

/// Nom du dernier dossier d'un chemin (« Téléchargements »), ou « Dossier personnel » pour la racine.
fn folder_name(path: &str) -> String {
    let leaf = path.trim_matches('/').rsplit('/').next().unwrap_or("");
    if leaf.is_empty() { "Dossier personnel".to_string() } else { leaf.to_string() }
}

/// Dossiers surveillés : « Téléchargements », « Images (+ sous-dossiers) ».
fn watched(rule: &Value) -> String {
    let items = rule["watch"].as_array().cloned().unwrap_or_default();
    let names: Vec<String> = items
        .iter()
        .map(|w| format!("{}{}", folder_name(w["path"].as_str().unwrap_or("")), if w["recursive"] == true { " (+ sous-dossiers)" } else { "" }))
        .collect();
    if names.is_empty() { "aucun dossier".to_string() } else { names.join(", ") }
}

/// Déclencheurs : « un fichier apparaît, à la demande seulement ».
fn triggers(rule: &Value) -> String {
    let items = rule["triggers"].as_array().cloned().unwrap_or_default();
    let names: Vec<String> = items.iter().map(|t| trigger_label(t["type"].as_str().unwrap_or(""))).collect();
    if names.is_empty() { "aucun déclencheur".to_string() } else { names.join(", ") }
}

/// Pluriel simple : « 1 condition », « 3 conditions ».
fn count(n: usize, word: &str) -> String {
    format!("{n} {word}{}", if n > 1 { "s" } else { "" })
}

/// Résumé d'une règle : `Surveille X · quand Y · 2 conditions → 1 action`.
pub fn summarize(rule: &Value) -> String {
    let conditions: usize = rule["alternatives"].as_array().map_or(0, |a| a.iter().map(|g| g["conditions"].as_array().map_or(0, Vec::len)).sum());
    let actions = rule["actions"].as_array().map_or(0, Vec::len);
    format!("Surveille {} · quand {} · {} → {}", watched(rule), triggers(rule), count(conditions, "condition"), count(actions, "action"))
}
