//! Barre du bas : raccourcis vers les dossiers et fichiers épinglés par l'utilisateur.

use gtk4::prelude::*;
use gtk4::{Box as GtkBox, PolicyType, ScrolledWindow};

use crate::ui::widgets::hbox;

/// Barre du bas : le conteneur des raccourcis, et la barre complète à poser contre le bord de la fenêtre.
pub fn build_dock() -> (GtkBox, GtkBox) {
    let items = hbox(6);
    items.set_valign(gtk4::Align::Center);
    let scroll = ScrolledWindow::builder().hscrollbar_policy(PolicyType::External).vscrollbar_policy(PolicyType::Never).child(&items).build();
    scroll.set_hexpand(true);
    let root = hbox(0);
    root.add_css_class("dock");
    root.append(&scroll);
    (root, items)
}
