//! Barre du bas choisie par l'utilisateur : dossiers et fichiers épinglés, lecture et ordre.

use serde_json::Value;

/// Lit la liste enregistrée ; les entrées qui ne sont pas des chemins non vides sont ignorées.
pub fn parse_dock(value: &Value) -> Vec<String> {
    let items = value.as_array().map(Vec::as_slice).unwrap_or_default();
    items.iter().filter_map(Value::as_str).filter(|p| !p.is_empty()).map(String::from).collect()
}

/// Ajoute un chemin en fin de barre ; `false` s'il y est déjà.
pub fn pin_dock(list: &mut Vec<String>, path: &str) -> bool {
    if list.iter().any(|p| p == path) {
        return false;
    }
    list.push(path.to_string());
    true
}

/// Retire un chemin de la barre ; `false` s'il n'y était pas.
pub fn unpin_dock(list: &mut Vec<String>, path: &str) -> bool {
    let before = list.len();
    list.retain(|p| p != path);
    list.len() != before
}
