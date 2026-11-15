//! Polices de l'interface, embarquées dans le binaire et déclarées à Pango au démarrage.

use std::fs;
use std::path::PathBuf;

use gtk4::prelude::*;
use pango::prelude::FontMapExt;

const FONTS: [(&str, &[u8]); 8] = [
    ("Anton-Regular.ttf", include_bytes!("../../assets/fonts/Anton-Regular.ttf")),
    ("HankenGrotesk-Regular.ttf", include_bytes!("../../assets/fonts/HankenGrotesk-Regular.ttf")),
    ("HankenGrotesk-Medium.ttf", include_bytes!("../../assets/fonts/HankenGrotesk-Medium.ttf")),
    ("HankenGrotesk-SemiBold.ttf", include_bytes!("../../assets/fonts/HankenGrotesk-SemiBold.ttf")),
    ("HankenGrotesk-Bold.ttf", include_bytes!("../../assets/fonts/HankenGrotesk-Bold.ttf")),
    ("JetBrainsMono-Regular.ttf", include_bytes!("../../assets/fonts/JetBrainsMono-Regular.ttf")),
    ("JetBrainsMono-Medium.ttf", include_bytes!("../../assets/fonts/JetBrainsMono-Medium.ttf")),
    ("JetBrainsMono-Bold.ttf", include_bytes!("../../assets/fonts/JetBrainsMono-Bold.ttf")),
];

/// Dossier de cache où les polices sont écrites pour que fontconfig les lise.
fn cache_dir() -> PathBuf {
    let base = std::env::var_os("XDG_CACHE_HOME").map(PathBuf::from).or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".cache")));
    base.unwrap_or_else(std::env::temp_dir).join("five/fonts")
}

/// Écrit les polices manquantes dans le cache et retourne leurs chemins.
fn unpack() -> Vec<PathBuf> {
    let dir = cache_dir();
    let _ = fs::create_dir_all(&dir);
    FONTS
        .iter()
        .filter_map(|(name, data)| {
            let path = dir.join(name);
            let fresh = fs::metadata(&path).is_ok_and(|m| m.len() == data.len() as u64);
            (fresh || fs::write(&path, data).is_ok()).then_some(path)
        })
        .collect()
}

/// Déclare Anton, Hanken Grotesk et JetBrains Mono à Pango pour tous les widgets du widget donné.
pub fn register(widget: &impl IsA<gtk4::Widget>) {
    let Some(map) = widget.pango_context().font_map() else { return };
    for path in unpack() {
        let _ = map.add_font_file(&path);
    }
}
