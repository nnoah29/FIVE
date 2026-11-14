//! Champs composés : comparaison de texte, durée, liste de conditions imbriquées.

use std::rc::Rc;

use gtk4::Box as GtkBox;
use gtk4::prelude::*;
use serde_json::{Value, json};

use super::fields_simple::{bool_field, list_field, number_field, text_field};
use super::select::{Choice, select_button};
use super::state::Editor;
use crate::model::rules_draft::{Seg, at_mut, key};
use crate::model::rules_fields_labels::choice_label;
use crate::ui::widgets::{hbox, label};

const OPS: [&str; 7] = ["contains", "equals", "starts_with", "ends_with", "glob", "regex", "in_list"];
const UNITS: [&str; 6] = ["seconds", "minutes", "hours", "days", "weeks", "months"];

/// Sélecteur d'une liste de valeurs, libellées en français.
fn options(values: &[&str]) -> Vec<Choice> {
    values.iter().map(|v| (v.to_string(), choice_label(v))).collect()
}

/// Comparaison de texte : opérateur, valeur (ou liste pour « est dans la liste »), respect de la casse.
pub fn matcher_field(ed: &Rc<Editor>, path: Vec<Seg>, value: &Value) -> GtkBox {
    let row = hbox(8);
    let op = value["op"].as_str().unwrap_or("contains").to_string();
    let op_path = [path.clone(), vec![key("op")]].concat();
    row.append(&select_button(ed, &op, options(&OPS), move |ed, picked| {
        ed.mutate(|draft| {
            if let Some(slot) = at_mut(draft, &op_path) {
                *slot = json!(picked);
            }
        })
    }));
    if op == "in_list" {
        row.append(&list_field(ed, [path.clone(), vec![key("values")]].concat(), value["values"].as_array().map_or(&[][..], Vec::as_slice)));
    } else {
        row.append(&text_field(ed, [path.clone(), vec![key("value")]].concat(), value["value"].as_str().unwrap_or("")));
    }
    row.append(&label("Casse", &["rule-field-label"]));
    row.append(&bool_field(ed, [path, vec![key("case_sensitive")]].concat(), value["case_sensitive"] == true));
    row
}

/// Durée : un nombre et une unité.
pub fn duration_field(ed: &Rc<Editor>, path: Vec<Seg>, value: &Value) -> GtkBox {
    let row = hbox(8);
    row.append(&number_field(ed, [path.clone(), vec![key("n")]].concat(), &value["n"]));
    let unit_path = [path, vec![key("unit")]].concat();
    row.append(&select_button(ed, value["unit"].as_str().unwrap_or("days"), options(&UNITS), move |ed, picked| ed.set(&unit_path, json!(picked))));
    row
}
