//! Thème effectif : réglage de l'utilisateur ou préférence du système.

use gtk4::glib;
use gtk4::prelude::*;

use crate::model::settings::Settings;

/// Indique si le système demande un thème sombre.
pub fn system_is_dark() -> bool {
    let Some(settings) = gtk4::Settings::default() else { return true };
    let scheme = settings.find_property("gtk-interface-color-scheme").and_then(|_| {
        let value = settings.property_value("gtk-interface-color-scheme");
        glib::EnumValue::from_value(&value).map(|(_, found)| found.value())
    });
    match scheme {
        Some(2) => true,
        Some(3) => false,
        _ => settings.is_gtk_application_prefer_dark_theme() || settings.gtk_theme_name().is_some_and(|n| n.to_lowercase().contains("dark")),
    }
}

/// Indique si le thème effectif est sombre, d'après le réglage de l'utilisateur.
pub fn is_dark(settings: &Settings) -> bool {
    match settings.theme.as_str() {
        "dark" => true,
        "light" => false,
        _ => system_is_dark(),
    }
}
