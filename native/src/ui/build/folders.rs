//! Section des dossiers : titre, fil d'Ariane, cartes défilantes.

use gtk4::prelude::*;
use gtk4::{PolicyType, ScrolledWindow};

use crate::ui::handles::FoldersUi;
use crate::ui::widgets::{hbox, label, vbox};

/// Section du haut de la page, sans contenu.
pub fn build_folders() -> FoldersUi {
    let root = vbox(0);
    root.add_css_class("folders");
    let title = label("Accueil", &["h1"]);
    title.set_ellipsize(gtk4::pango::EllipsizeMode::End);
    title.set_halign(gtk4::Align::Fill);
    let crumbs = hbox(0);
    crumbs.add_css_class("crumbs");
    let cards = hbox(12);
    let scroll = ScrolledWindow::builder().hscrollbar_policy(PolicyType::Automatic).vscrollbar_policy(PolicyType::Never).child(&cards).build();
    scroll.add_css_class("cards");
    root.append(&title);
    root.append(&crumbs);
    root.append(&scroll);
    FoldersUi { root, title, crumbs, cards }
}
