//! Contrôles de formulaire : case à cocher carrée, bouton d'un groupe segmenté.

use gtk4::Button;
use gtk4::prelude::*;

use super::classes::{centered, set_class};
use super::icon::icon_with_stroke;

/// Case à cocher carrée, cochée ou non.
pub fn tick(on: bool) -> Button {
    let button = Button::new();
    button.add_css_class("tick");
    button.set_hexpand(false);
    button.set_halign(gtk4::Align::Center);
    let frame = centered(14, 14, &icon_with_stroke("check", 10, 2.6));
    frame.add_css_class("tick-box");
    frame.set_halign(gtk4::Align::Center);
    frame.set_valign(gtk4::Align::Center);
    button.set_child(Some(&frame));
    set_tick(&button, on);
    button
}

/// Met la case dans l'état voulu.
pub fn set_tick(button: &Button, on: bool) {
    set_class(button, "on", on);
}

/// Indique si la case est cochée.
pub fn is_ticked(button: &Button) -> bool {
    button.has_css_class("on")
}
