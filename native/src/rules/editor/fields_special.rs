//! Champs spéciaux de l'éditeur : taille, droits en octal, date, dossier avec sélecteur.

use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Entry, gio};
use serde_json::{Value, json};

use super::state::Editor;
use crate::app::dialog::field;
use crate::model::rules_draft::Seg;
use crate::model::rules_values::{date_text, format_bytes, parse_bytes, parse_date, parse_octal};
use crate::ui::classes::set_class;
use crate::ui::widgets::{hbox, text_button};

/// Champ dont la saisie est convertie en JSON par `parse` ; une saisie illisible garde l'ancienne valeur et entoure le champ.
fn parsed_field(ed: &Rc<Editor>, path: Vec<Seg>, shown: &str, parse: impl Fn(&str) -> Option<Value> + 'static) -> Entry {
    let entry = field(shown);
    entry.set_width_chars(16);
    let weak = Rc::downgrade(ed);
    entry.connect_changed(move |e| {
        let value = parse(&e.text());
        set_class(e, "invalid", value.is_none() && !e.text().is_empty());
        if let (Some(value), Some(ed)) = (value, weak.upgrade()) {
            ed.set(&path, value);
        }
    });
    entry
}

/// Taille saisie « 5 Mo », stockée en octets.
pub fn bytes_field(ed: &Rc<Editor>, path: Vec<Seg>, value: &Value) -> Entry {
    parsed_field(ed, path, &value.as_u64().map_or(String::new(), format_bytes), |t| parse_bytes(t).map(|b| json!(b)))
}

/// Droits saisis en octal (`644`), stockés en nombre.
pub fn octal_field(ed: &Rc<Editor>, path: Vec<Seg>, value: &Value) -> Entry {
    parsed_field(ed, path, &value.as_u64().map_or(String::new(), |m| format!("{m:o}")), |t| parse_octal(t).map(|m| json!(m)))
}

/// Date saisie `AAAA-MM-JJ hh:mm`, stockée en secondes Unix.
pub fn date_field(ed: &Rc<Editor>, path: Vec<Seg>, value: &Value) -> Entry {
    parsed_field(ed, path, &date_text(value.as_f64()), |t| parse_date(t).map(|s| json!(s)))
}

/// Ouvre le sélecteur de dossier et écrit le chemin relatif à la racine dans le champ.
fn browse(ed: &Rc<Editor>, entry: &Entry) {
    let Some(app) = ed.app() else { return };
    let dialog = gtk4::FileDialog::new();
    dialog.set_initial_folder(Some(&gio::File::for_path(app.root.path())));
    let (entry, root) = (entry.clone(), app.root.clone());
    dialog.select_folder(Some(&app.ui.window), None::<&gio::Cancellable>, move |result| {
        let Some(chosen) = result.ok().and_then(|f| f.path()) else { return };
        if root.inside(&chosen) {
            entry.set_text(&root.relative(&chosen));
        }
    });
}

/// Dossier relatif à la racine, saisi ou choisi avec le sélecteur de dossier.
pub fn folder_field(ed: &Rc<Editor>, path: Vec<Seg>, value: &str) -> GtkBox {
    let row = hbox(8);
    let entry = field(value);
    entry.set_hexpand(true);
    let weak = Rc::downgrade(ed);
    entry.connect_changed(move |e| {
        if let Some(ed) = weak.upgrade() {
            ed.set(&path, json!(e.text().trim().trim_matches('/').to_string()));
        }
    });
    let pick = text_button("Choisir…", Some("folder"), &[]);
    let (weak, target) = (Rc::downgrade(ed), entry.clone());
    pick.connect_clicked(move |_| {
        if let Some(ed) = weak.upgrade() {
            browse(&ed, &target);
        }
    });
    row.append(&entry);
    row.append(&pick);
    row
}
