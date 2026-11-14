//! Lecture des couleurs du fond d'écran écrites par le shell du bureau (`/tmp/qs_colors.json`).

use std::env;
use std::fs;
use std::time::SystemTime;

use crate::model::settings::Settings;
use crate::model::wall_seed::{WallSeed, parse_seed};

/// Chemin du fichier JSON de couleurs ; `QS_COLORS_FILE` le remplace.
fn json_path() -> String {
    env::var("QS_COLORS_FILE").unwrap_or_else(|_| "/tmp/qs_colors.json".into())
}

/// Lit les couleurs du fond d'écran ; `None` si le réglage est coupé ou le fichier absent.
pub fn wall_seed(settings: &Settings) -> Option<WallSeed> {
    if !settings.wallpaper {
        return None;
    }
    let json = fs::read_to_string(json_path()).ok()?;
    let home = env::var("HOME").unwrap_or_default();
    let css = fs::read_to_string(format!("{home}/.cache/theme-color/colors.css")).unwrap_or_default();
    parse_seed(&json, &css)
}

/// Date de dernière écriture du fichier de couleurs, pour détecter un changement de fond d'écran.
pub fn wall_stamp() -> Option<SystemTime> {
    fs::metadata(json_path()).and_then(|m| m.modified()).ok()
}
