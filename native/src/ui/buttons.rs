//! Boutons à icône : bouton carré et bouton « ⋮ ».

use gtk4::prelude::*;
use gtk4::{Align, Button};

use super::icon::icon;
use super::widgets::text_button;

/// Bouton carré qui ne porte qu'une icône.
pub fn icon_button(icon_name: &str, classes: &[&str]) -> Button {
    let button = text_button("", Some(icon_name), classes);
    button.add_css_class("icon");
    button
}

/// Bouton « ⋮ » de 30 pixels qui ouvre le menu d'un élément.
pub fn dots_button() -> Button {
    let button = Button::new();
    button.add_css_class("dots");
    button.set_child(Some(&icon("more")));
    button.set_size_request(30, 30);
    button.set_hexpand(false);
    button.set_halign(Align::Center);
    button.set_valign(Align::Center);
    button
}
