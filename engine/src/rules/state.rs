//! État persistant des règles (`etat-regles.json`) : fichiers suivis, dernières exécutions, plafond de notifications.

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// Dernier aspect connu d'un fichier surveillé : taille, date, depuis quand il ne bouge plus.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Tracked {
    pub size: u64,
    pub mtime: f64,
    pub since: f64,
    #[serde(default)]
    pub done: bool,
}

/// Mémoire entre deux passes : sans elle, les fichiers déjà présents seraient retraités à chaque démarrage.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RulesState {
    #[serde(default)]
    pub tracked: BTreeMap<String, Tracked>,
    #[serde(default)]
    pub last_run: BTreeMap<String, f64>,
    #[serde(default)]
    pub notify_day: String,
    #[serde(default)]
    pub notify_count: u32,
    #[serde(default)]
    pub disks: Vec<String>,
    #[serde(default)]
    pub baselined: bool,
}

/// Emplacement du fichier d'état dans le dossier de configuration.
pub fn state_path(dir: &Path) -> PathBuf {
    dir.join("etat-regles.json")
}

/// Lit l'état ; un fichier absent ou abîmé redonne un état vide (les fichiers présents seront alors simplement mémorisés).
pub fn load_state(path: &Path) -> RulesState {
    fs::read_to_string(path).ok().and_then(|text| serde_json::from_str(&text).ok()).unwrap_or_default()
}

/// Écrit l'état : fichier temporaire puis renommage.
pub fn save_state(path: &Path, state: &RulesState) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let temporary = path.with_extension("json.tmp");
    fs::write(&temporary, serde_json::to_vec(state)?)?;
    fs::rename(temporary, path)
}
