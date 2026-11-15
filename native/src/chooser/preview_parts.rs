//! Informations affichées sous l'aperçu : nom, type, taille ou contenu, date.

use std::fs;

use five_engine::entry::Entry;
use gtk4::prelude::*;

use super::listing::Row;
use crate::model::format::{format_date, format_size};
use crate::ui::widgets::{label, vbox};

/// Description du moteur pour une ligne : ce qu'il faut à l'aperçu partagé avec la fenêtre principale.
pub fn entry_of(row: &Row) -> Entry {
    Entry {
        name: row.name.clone(),
        is_dir: row.is_dir,
        size: row.size,
        modified: row.modified,
        is_link: false,
        mime: (!row.is_dir).then(|| five_engine::mime::guess(&row.name).to_string()),
        path: row.path.to_string_lossy().into_owned(),
        children: None,
    }
}

/// Informations affichées sous l'aperçu : type, taille ou contenu, date.
pub fn facts_box(row: &Row) -> gtk4::Box {
    let column = vbox(4);
    let name = label(&row.name, &["chooser-fact-name"]);
    name.set_ellipsize(gtk4::pango::EllipsizeMode::Middle);
    name.set_max_width_chars(28);
    name.set_tooltip_text(Some(&row.name));
    let size = if row.is_dir { fs::read_dir(&row.path).map_or(String::new(), |d| format!("{} éléments", d.count())) } else { format_size(row.size, false) };
    let kind = if row.is_dir { "Dossier" } else { five_engine::mime::guess(&row.name) };
    column.append(&name);
    for text in [kind.to_string(), size, format_date(row.modified, true)] {
        column.append(&label(&text, &["dim", "chooser-fact"]));
    }
    column
}
