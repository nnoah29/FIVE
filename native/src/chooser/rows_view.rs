//! Lignes de la liste du sélecteur : icône, nom, taille, date.

use gtk4::prelude::*;
use gtk4::{Align, ListBox, ListBoxRow};

use super::listing::Row;
use crate::model::format::{format_date, format_size};
use crate::ui::icon::icon;
use crate::ui::widgets::{hbox, label};

/// Nom du fichier, tronqué au milieu pour garder l'extension visible.
fn name_label(row: &Row) -> gtk4::Label {
    let name = label(&row.name, &[]);
    name.set_ellipsize(gtk4::pango::EllipsizeMode::Middle);
    name.set_hexpand(true);
    name.set_halign(Align::Fill);
    name
}

/// Construit le contenu d'une ligne.
fn row_content(row: &Row) -> gtk4::Box {
    let content = hbox(12);
    content.add_css_class("chooser-row");
    content.append(&icon(if row.is_dir { "folder" } else { "file" }));
    content.append(&name_label(row));
    let size = if row.is_dir { String::new() } else { format_size(row.size, false) };
    let (size, date) = (label(&size, &["dim", "chooser-size"]), label(&format_date(row.modified, false), &["dim", "chooser-date"]));
    size.set_xalign(1.0);
    date.set_xalign(1.0);
    content.append(&size);
    content.append(&date);
    content
}

/// Remplace le contenu de la liste par les lignes données.
pub fn fill(list: &ListBox, rows: &[Row]) {
    while let Some(child) = list.first_child() {
        list.remove(&child);
    }
    for row in rows {
        let item = ListBoxRow::new();
        item.set_child(Some(&row_content(row)));
        list.append(&item);
    }
}
