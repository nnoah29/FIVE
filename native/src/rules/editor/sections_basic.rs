//! Sections de base de l'éditeur : nom, dossiers surveillés, bouton d'ajout.

use std::rc::Rc;

use gtk4::Box as GtkBox;
use gtk4::prelude::*;
use serde_json::{Value, json};

use super::conditions::section_box;
use super::fields_simple::{choice_field, text_field};
use super::state::Editor;
use super::watch::watch_row;
use crate::model::rules_draft::{at_mut, key};
use crate::ui::widgets::{hbox, label, text_button};

/// Ajoute `fresh` à la fin du tableau situé sous la clé `name` du brouillon.
pub fn push(ed: &Rc<Editor>, name: &'static str, fresh: Value) {
    ed.mutate(|d| {
        if let Some(items) = at_mut(d, &[key(name)]).and_then(|v| v.as_array_mut()) {
            items.push(fresh);
        }
    });
}

/// Bouton « Ajouter … » d'une section.
pub fn add_button(ed: &Rc<Editor>, text: &str, name: &'static str, fresh: impl Fn(&Editor) -> Value + 'static) -> gtk4::Button {
    let button = text_button(text, Some("plus"), &[]);
    let weak = Rc::downgrade(ed);
    button.connect_clicked(move |_| {
        if let Some(ed) = weak.upgrade() {
            let value = fresh(&ed);
            push(&ed, name, value);
        }
    });
    button
}

/// Nom de la règle et politique quand la destination existe déjà.
pub fn header_section(ed: &Rc<Editor>) -> GtkBox {
    let block = section_box("Règle");
    let name = ed.get(&[key("name")]);
    block.append(&text_field(ed, vec![key("name")], name.as_str().unwrap_or("")));
    let row = hbox(12);
    row.append(&label("SI LE NOM EXISTE DÉJÀ", &["rule-field-label"]));
    let current = ed.get(&[key("on_conflict")]);
    let choices: Vec<String> = ["rename", "skip", "ask"].iter().map(|s| s.to_string()).collect();
    row.append(&choice_field(ed, vec![key("on_conflict")], &choices, current.as_str().unwrap_or("rename")));
    block.append(&row);
    block
}

/// Dossiers surveillés.
pub fn watch_section(ed: &Rc<Editor>) -> GtkBox {
    let block = section_box("Dossiers surveillés");
    let items = ed.get(&[key("watch")]).as_array().cloned().unwrap_or_default();
    items.iter().enumerate().for_each(|(index, watch)| block.append(&watch_row(ed, index, watch)));
    block.append(&add_button(ed, "Ajouter un dossier", "watch", |_| json!({"path": "", "recursive": false, "ignore_hidden": true})));
    block
}
