//! Apparence du sélecteur : mêmes réglages, thème et polices que l'application.

use std::path::PathBuf;

use gtk4::{CssProvider, Window, gdk};

use crate::model::settings::Settings;
use crate::ui::fonts;
use crate::ui::scheme::is_dark;
use crate::ui::theme::build_css;
use crate::ui::wallpaper::wall_seed;

/// Applique le thème de FIVE à la fenêtre du sélecteur.
pub fn apply(window: &Window) {
    let home = std::env::var_os("HOME").map(PathBuf::from).unwrap_or_default();
    let settings = Settings::load(&home);
    let provider = CssProvider::new();
    let wall = wall_seed(&settings);
    provider.load_from_string(&build_css(is_dark(&settings), &settings, wall.as_ref()));
    if let Some(display) = gdk::Display::default() {
        gtk4::style_context_add_provider_for_display(&display, &provider, gtk4::STYLE_PROVIDER_PRIORITY_USER);
    }
    fonts::register(window);
}
