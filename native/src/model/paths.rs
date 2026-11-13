//! Chemins relatifs à la racine : jointure, parent, corbeille, nom et extension.

use five_engine::places::TRASH;

/// Joint deux morceaux de chemin relatif.
pub fn join_path(base: &str, name: &str) -> String {
    if base.is_empty() { name.to_string() } else { format!("{base}/{name}") }
}

/// Dossier parent d'un chemin relatif (vide pour la racine).
pub fn parent_of(path: &str) -> String {
    path.rsplit_once('/').map_or_else(String::new, |(parent, _)| parent.to_string())
}

/// Indique si le chemin est la corbeille ou en dépend.
pub fn in_trash(path: &str) -> bool {
    path == TRASH || path.starts_with(&format!("{TRASH}/"))
}

/// Sépare un nom en radical et extension (avec le point) ; un nom caché n'a pas d'extension.
pub fn name_parts(name: &str) -> (String, String) {
    match name.rfind('.') {
        Some(at) if at > 0 => (name[..at].to_string(), name[at..].to_string()),
        _ => (name.to_string(), String::new()),
    }
}

/// Dernier composant d'un chemin relatif.
pub fn leaf(path: &str) -> String {
    path.rsplit('/').next().unwrap_or("").to_string()
}
