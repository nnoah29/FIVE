//! Dernier emplacement consulté, pour reprendre où l'on s'est arrêté.

use std::fs;
use std::path::Path;

use serde_json::{Value, json};

use super::settings::config_dir;

/// Lit le dernier emplacement : chemin relatif et vue virtuelle éventuelle.
pub fn load_session(home: &Path) -> (String, Option<String>) {
    let text = fs::read_to_string(config_dir(home).join("session.json")).unwrap_or_default();
    let json: Value = serde_json::from_str(&text).unwrap_or(Value::Null);
    (json["path"].as_str().unwrap_or("").to_string(), json["virtual"].as_str().map(String::from))
}

/// Enregistre l'emplacement courant ; un échec d'écriture est sans conséquence.
pub fn save_session(home: &Path, path: &str, virtual_view: Option<&str>) {
    let dir = config_dir(home);
    if fs::create_dir_all(&dir).is_ok() {
        let _ = fs::write(dir.join("session.json"), json!({"path": path, "virtual": virtual_view}).to_string());
    }
}
