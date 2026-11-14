//! Une ligne de formulaire : libellé du champ et le bon contrôle selon le genre décrit par le catalogue.

use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Widget};
use serde_json::Value;

use super::conditions::conditions_list;
use super::fields_compound::{duration_field, matcher_field};
use super::fields_multi::multi_field;
use super::fields_simple::{bool_field, choice_field, list_field, number_field, numbers_field, text_field};
use super::fields_special::{bytes_field, date_field, folder_field, octal_field};
use super::state::Editor;
use crate::model::rules_draft::{Seg, key};
use crate::model::rules_fields_labels::field_label;
use crate::ui::widgets::{hbox, label};

/// Choix permis d'un champ, ou liste vide.
fn choices_of(descriptor: &Value) -> Vec<String> {
    descriptor["choices"].as_array().map(|c| c.iter().filter_map(|v| v.as_str().map(String::from)).collect()).unwrap_or_default()
}

/// Contrôle d'un champ selon son genre.
fn control(ed: &Rc<Editor>, descriptor: &Value, path: Vec<Seg>, value: &Value) -> Widget {
    let (choices, items) = (choices_of(descriptor), value.as_array().cloned().unwrap_or_default());
    match descriptor["kind"].as_str().unwrap_or("text") {
        "matcher" => matcher_field(ed, path, value).upcast(),
        "duration" => duration_field(ed, path, value).upcast(),
        "conditions" => conditions_list(ed, path).upcast(),
        "date" => date_field(ed, path, value).upcast(),
        "bytes" => bytes_field(ed, path, value).upcast(),
        "octal" => octal_field(ed, path, value).upcast(),
        "folder" => folder_field(ed, path, value.as_str().unwrap_or("")).upcast(),
        "bool" => bool_field(ed, path, value == &Value::Bool(true)).upcast(),
        "number" => number_field(ed, path, value).upcast(),
        "numbers" => numbers_field(ed, path, &items).upcast(),
        "list" if !choices.is_empty() => multi_field(ed, path, &choices, &items).upcast(),
        "list" => list_field(ed, path, &items).upcast(),
        _ if !choices.is_empty() => choice_field(ed, path, &choices, value.as_str().unwrap_or("")).upcast(),
        _ => text_field(ed, path, value.as_str().unwrap_or("")).upcast(),
    }
}

/// Ligne « libellé — contrôle » du champ `descriptor` de l'élément `item`, situé sous `base`.
pub fn field_row(ed: &Rc<Editor>, base: &[Seg], descriptor: &Value, item: &Value) -> GtkBox {
    let name = descriptor["name"].as_str().unwrap_or("");
    let kind = descriptor["kind"].as_str().unwrap_or("");
    let row = hbox(12);
    row.append(&label(&field_label(name, kind).to_uppercase(), &["rule-field-label"]));
    let widget = control(ed, descriptor, [base.to_vec(), vec![key(name)]].concat(), item.get(name).unwrap_or(&Value::Null));
    widget.set_hexpand(true);
    row.append(&widget);
    row
}
