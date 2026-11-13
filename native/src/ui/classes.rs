//! Petits outils sur les widgets : classes CSS, vidage de conteneur, largeur fixe.

use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Button, Label, Widget};

/// Ajoute ou retire une classe CSS selon `on`.
pub fn set_class(widget: &impl IsA<Widget>, class: &str, on: bool) {
    if on { widget.add_css_class(class) } else { widget.remove_css_class(class) }
}

/// Retire tous les enfants d'une boîte.
pub fn clear(container: &GtkBox) {
    while let Some(child) = container.first_child() {
        container.remove(&child);
    }
}

/// Place un widget dans un cadre de largeur fixe (colonnes alignées entre en-tête et lignes).
pub fn fixed(width: i32, child: &impl IsA<Widget>) -> GtkBox {
    let frame = GtkBox::new(gtk4::Orientation::Horizontal, 0);
    frame.set_size_request(width, -1);
    frame.set_hexpand(false);
    frame.set_valign(gtk4::Align::Center);
    frame.append(child);
    frame
}

/// Change le texte d'un bouton créé par `text_button` (le dernier libellé de sa rangée).
pub fn set_button_text(button: &Button, text: &str) {
    let label = button.child().and_then(|row| row.last_child()).and_downcast::<Label>();
    if let Some(label) = label {
        label.set_text(&text.to_uppercase());
    }
}

/// Cadre de taille fixe qui centre son contenu sans s'étirer avec lui.
pub fn centered(width: i32, height: i32, child: &impl IsA<Widget>) -> GtkBox {
    let frame = GtkBox::new(gtk4::Orientation::Horizontal, 0);
    frame.set_size_request(width, height);
    frame.set_hexpand_set(true);
    frame.set_hexpand(false);
    frame.set_vexpand_set(true);
    frame.set_vexpand(false);
    child.set_halign(gtk4::Align::Center);
    child.set_valign(gtk4::Align::Center);
    child.set_hexpand(true);
    frame.append(child);
    frame
}
