//! Réglages de l'utilisateur, enregistrés dans `~/.config/five/reglages.json`.

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{Value, json};

/// Réglages persistants de l'interface.
#[derive(Debug, Clone, PartialEq)]
pub struct Settings {
    pub theme: String,
    pub view: String,
    pub sort: String,
    pub ascending: bool,
    pub hidden: bool,
    pub binary_units: bool,
    pub full_dates: bool,
    pub thumbs: bool,
    pub confirm_trash: bool,
    pub accent: String,
    pub compact: bool,
    pub git: bool,
    pub restore: bool,
    pub wallpaper: bool,
}

impl Default for Settings {
    /// Valeurs d'origine de l'interface web.
    fn default() -> Self {
        Self {
            theme: "system".into(),
            view: "list".into(),
            sort: "n".into(),
            ascending: true,
            hidden: false,
            binary_units: false,
            full_dates: false,
            thumbs: true,
            confirm_trash: true,
            accent: String::new(),
            compact: false,
            git: true,
            restore: true,
            wallpaper: true,
        }
    }
}

/// Dossier de configuration de FIVE sous `home`.
pub fn config_dir(home: &Path) -> PathBuf {
    home.join(".config/five")
}

impl Settings {
    /// Lit les réglages ; un fichier absent ou abîmé redonne les valeurs d'origine.
    pub fn load(home: &Path) -> Settings {
        let text = fs::read_to_string(config_dir(home).join("reglages.json")).unwrap_or_default();
        let json: Value = serde_json::from_str(&text).unwrap_or(Value::Null);
        Settings::from_json(&json)
    }

    /// Construit les réglages depuis un JSON, champ par champ.
    pub fn from_json(json: &Value) -> Settings {
        let base = Settings::default();
        let text = |key: &str, fallback: &str| json[key].as_str().unwrap_or(fallback).to_string();
        let flag = |key: &str, fallback: bool| json[key].as_bool().unwrap_or(fallback);
        Settings {
            theme: text("theme", &base.theme),
            view: text("view", &base.view),
            sort: text("sort", &base.sort),
            ascending: flag("ascending", base.ascending),
            hidden: flag("hidden", base.hidden),
            binary_units: flag("binary_units", base.binary_units),
            full_dates: flag("full_dates", base.full_dates),
            thumbs: flag("thumbs", base.thumbs),
            confirm_trash: flag("confirm_trash", base.confirm_trash),
            accent: text("accent", &base.accent),
            compact: flag("compact", base.compact),
            git: flag("git", base.git),
            restore: flag("restore", base.restore),
            wallpaper: flag("wallpaper", base.wallpaper),
        }
    }

    /// Représentation JSON enregistrée sur disque.
    pub fn to_json(&self) -> Value {
        json!({"theme": self.theme, "view": self.view, "sort": self.sort, "ascending": self.ascending, "hidden": self.hidden,
            "binary_units": self.binary_units, "full_dates": self.full_dates, "thumbs": self.thumbs, "confirm_trash": self.confirm_trash,
            "accent": self.accent, "compact": self.compact, "git": self.git, "restore": self.restore, "wallpaper": self.wallpaper})
    }

    /// Enregistre les réglages ; un échec d'écriture est sans conséquence.
    pub fn save(&self, home: &Path) {
        let dir = config_dir(home);
        if fs::create_dir_all(&dir).is_ok() {
            let _ = fs::write(dir.join("reglages.json"), self.to_json().to_string());
        }
    }
}
