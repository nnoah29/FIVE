//! Section des fichiers : titre, filtres, bascule liste/grille, en-tête de colonnes, liste.

use gtk4::prelude::*;
use gtk4::{PolicyType, ScrolledWindow};

use crate::ui::buttons::icon_button;
use crate::ui::handles::FilesUi;
use crate::ui::widgets::{hbox, label, text_button, vbox};

/// Bascule liste/grille : deux boutons dans un cadre.
fn view_switch() -> (gtk4::Box, gtk4::Button, gtk4::Button) {
    let frame = hbox(0);
    frame.add_css_class("seg");
    let (list, grid) = (icon_button("list", &["seg-btn"]), icon_button("grid", &["seg-btn"]));
    for button in [&list, &grid] {
        button.remove_css_class("btn");
        button.remove_css_class("icon");
        frame.append(button);
    }
    (frame, list, grid)
}

/// En-tête de la section : titre, filtres, coller, bascule d'affichage.
fn head() -> (gtk4::Box, gtk4::Label, gtk4::Box, gtk4::Button, gtk4::Button, gtk4::Button) {
    let row = hbox(16);
    row.add_css_class("files-head");
    let title = label("Tous les fichiers", &["h2"]);
    title.set_hexpand(true);
    let pills = hbox(6);
    let paste = text_button("Coller", Some("paste"), &[]);
    paste.set_visible(false);
    let (frame, list, grid) = view_switch();
    for widget in [title.upcast_ref::<gtk4::Widget>(), pills.upcast_ref(), paste.upcast_ref(), frame.upcast_ref()] {
        row.append(widget);
    }
    (row, title, pills, paste, list, grid)
}

/// Section des fichiers complète, sans contenu.
pub fn build_files() -> FilesUi {
    let root = vbox(0);
    root.add_css_class("files");
    let (row, title, pills, paste, view_list, view_grid) = head();
    let thead = hbox(10);
    thead.add_css_class("thead");
    let items = vbox(0);
    let scroll = ScrolledWindow::builder().hscrollbar_policy(PolicyType::Never).vexpand(true).child(&items).build();
    root.append(&row);
    root.append(&thead);
    root.append(&scroll);
    FilesUi { root, title, pills, paste, view_list, view_grid, thead, items, scroll }
}
