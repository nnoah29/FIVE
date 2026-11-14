//! Dossiers de la barre latérale choisis par l'utilisateur : lecture, écriture, ordre, épinglage.

use serde_json::{Value, json};

const ICONS: [&str; 7] = ["folder", "desktop", "downloads", "docs", "pictures", "music", "videos"];

/// Dossier épinglé : nom affiché (vide = nom du dossier), chemin absolu, icône.
#[derive(Debug, Clone, PartialEq)]
pub struct SideFolder {
    pub label: String,
    pub path: String,
    pub icon: String,
}

/// Lit la liste enregistrée ; `None` si elle est absente (mode automatique), entrées sans chemin ignorées, icône inconnue remplacée par un dossier.
pub fn parse_folders(value: &Value) -> Option<Vec<SideFolder>> {
    let text = |item: &Value, key: &str| item[key].as_str().unwrap_or("").to_string();
    let list = value.as_array()?;
    let kept = list.iter().map(|item| SideFolder { label: text(item, "label"), path: text(item, "path"), icon: text(item, "icon") });
    Some(
        kept.filter(|f| !f.path.is_empty())
            .map(|f| SideFolder { icon: if ICONS.contains(&f.icon.as_str()) { f.icon } else { "folder".into() }, ..f })
            .collect(),
    )
}

/// Représentation JSON enregistrée sur disque.
pub fn folders_to_json(list: &[SideFolder]) -> Value {
    Value::Array(list.iter().map(|f| json!({"label": f.label, "path": f.path, "icon": f.icon})).collect())
}

/// Décale l'élément `from` de `delta` rangs (-1 monte, 1 descend) ; sans effet hors de la liste.
pub fn move_folder(list: &mut [SideFolder], from: usize, delta: isize) {
    let to = from as isize + delta;
    if from < list.len() && to >= 0 && (to as usize) < list.len() {
        list.swap(from, to as usize);
    }
}

/// Ajoute un dossier en fin de liste ; `false` s'il y est déjà.
pub fn pin_folder(list: &mut Vec<SideFolder>, path: &str) -> bool {
    if list.iter().any(|f| f.path == path) {
        return false;
    }
    list.push(SideFolder { label: String::new(), path: path.to_string(), icon: "folder".into() });
    true
}

/// Nom affiché : le nom choisi, sinon le dernier composant du chemin.
pub fn shown_label(folder: &SideFolder) -> String {
    let name = folder.label.trim();
    if name.is_empty() { folder.path.trim_end_matches('/').rsplit('/').next().unwrap_or("").to_string() } else { name.to_string() }
}
