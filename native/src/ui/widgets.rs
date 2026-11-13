//! Fabriques de widgets : boîtes, libellés et boutons dans le style de l'interface.

use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Button, Label, Orientation};

use super::icon::icon;

/// Boîte horizontale.
pub fn hbox(spacing: i32) -> GtkBox {
    GtkBox::new(Orientation::Horizontal, spacing)
}

/// Boîte verticale.
pub fn vbox(spacing: i32) -> GtkBox {
    GtkBox::new(Orientation::Vertical, spacing)
}

/// Libellé aligné à gauche, avec ses classes CSS.
pub fn label(text: &str, classes: &[&str]) -> Label {
    let label = Label::new(Some(text));
    label.set_xalign(0.0);
    label.set_halign(Align::Start);
    classes.iter().for_each(|class| label.add_css_class(class));
    label
}

/// Libellé en petites capitales à chasse fixe (`mono`), mis en majuscules.
pub fn mono(text: &str, classes: &[&str]) -> Label {
    let label = label(&text.to_uppercase(), classes);
    label.add_css_class("mono");
    label
}

/// Bouton portant une icône facultative et un texte en majuscules à chasse fixe.
pub fn text_button(text: &str, icon_name: Option<&str>, classes: &[&str]) -> Button {
    let button = Button::new();
    let row = hbox(10);
    row.set_halign(Align::Center);
    if let Some(name) = icon_name {
        row.append(&icon(name));
    }
    if !text.is_empty() {
        row.append(&label(&text.to_uppercase(), &[]));
    }
    button.set_child(Some(&row));
    button.add_css_class("btn");
    classes.iter().for_each(|class| button.add_css_class(class));
    button
}

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
