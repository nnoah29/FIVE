//! En-tête du sélecteur : titre, dossier parent, chemin modifiable, fichiers cachés.

use gtk4::prelude::*;
use gtk4::{Button, Entry};

use crate::ui::buttons::icon_button;
use crate::ui::widgets::{hbox, label, vbox};

/// Éléments de l'en-tête que la fenêtre doit câbler.
pub struct Header {
    pub root: gtk4::Box,
    pub up: Button,
    pub path: Entry,
    pub hidden: Button,
}

/// Champ du chemin : large, texte à chasse fixe.
fn path_entry() -> Entry {
    let entry = Entry::new();
    entry.add_css_class("chooser-path");
    entry.set_hexpand(true);
    entry.set_placeholder_text(Some("Chemin du dossier"));
    entry
}

/// Construit l'en-tête : titre au-dessus, puis la barre de navigation.
pub fn build_header(title: &str) -> Header {
    let root = vbox(10);
    root.add_css_class("chooser-header");
    root.append(&label(&title.to_uppercase(), &["h2", "chooser-title"]));
    let (up, hidden, path) = (icon_button("up", &[]), icon_button("eye", &[]), path_entry());
    up.set_tooltip_text(Some("Dossier parent (Alt + ↑)"));
    hidden.set_tooltip_text(Some("Afficher les fichiers cachés"));
    let bar = hbox(8);
    bar.append(&up);
    bar.append(&path);
    bar.append(&hidden);
    root.append(&bar);
    Header { root, up, path, hidden }
}
