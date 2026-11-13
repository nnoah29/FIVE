//! Lecture de `/proc/mounts`.

use std::fs;

/// Un point de montage : périphérique, chemin, type de système de fichiers.
#[derive(Debug, Clone)]
pub struct Mount {
    pub device: String,
    pub point: String,
    pub fstype: String,
}

/// Décode les espaces échappés (`\040`) d'un chemin de montage.
fn unescape(point: &str) -> String {
    point.replace("\\040", " ")
}

/// Liste des montages ; vide si `/proc/mounts` est illisible.
pub fn read_mounts() -> Vec<Mount> {
    let text = fs::read_to_string("/proc/mounts").unwrap_or_default();
    text.lines()
        .filter_map(|line| {
            let mut parts = line.split_whitespace();
            Some(Mount { device: parts.next()?.to_string(), point: unescape(parts.next()?), fstype: parts.next()?.to_string() })
        })
        .collect()
}
