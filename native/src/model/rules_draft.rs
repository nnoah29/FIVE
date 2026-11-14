//! Brouillon d'une règle en cours d'édition : chemins dans le JSON, déplacement et retrait d'éléments, règle neuve.

use serde_json::{Value, json};

/// Étape d'un chemin dans le JSON : une clé d'objet ou un rang de tableau.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Seg {
    Key(String),
    Idx(usize),
}

/// Raccourci pour une clé.
pub fn key(name: &str) -> Seg {
    Seg::Key(name.to_string())
}

/// Valeur modifiable au bout du chemin ; `None` si le chemin n'existe pas.
pub fn at_mut<'a>(root: &'a mut Value, path: &[Seg]) -> Option<&'a mut Value> {
    path.iter().try_fold(root, |value, seg| match seg {
        Seg::Key(name) => value.get_mut(name.as_str()),
        Seg::Idx(index) => value.get_mut(*index),
    })
}

/// Décale l'élément `from` d'un tableau de `delta` rangs ; sans effet hors du tableau.
pub fn move_item(list: &mut Value, from: usize, delta: isize) {
    let Some(items) = list.as_array_mut() else { return };
    let to = from as isize + delta;
    if from < items.len() && to >= 0 && (to as usize) < items.len() {
        items.swap(from, to as usize);
    }
}

/// Retire l'élément `at` d'un tableau.
pub fn remove_item(list: &mut Value, at: usize) {
    if let Some(items) = list.as_array_mut().filter(|items| at < items.len()) {
        items.remove(at);
    }
}

/// Règle neuve : un dossier à choisir, « un fichier apparaît », une alternative vide, aucune action.
pub fn new_rule_draft() -> Value {
    json!({
        "name": "Nouvelle règle", "enabled": true, "trial": true,
        "watch": [{"path": "", "recursive": false, "ignore_hidden": true}],
        "triggers": [{"type": "created"}],
        "alternatives": [{"conditions": []}],
        "actions": [],
        "on_conflict": "rename"
    })
}
