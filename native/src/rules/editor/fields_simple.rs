//! Champs simples de l'éditeur : texte, nombre, case à cocher, choix, listes.

use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{Button, Entry};
use serde_json::{Value, json};

use super::select::{Choice, select_button};
use super::state::Editor;
use crate::app::dialog::field;
use crate::model::rules_draft::Seg;
use crate::model::rules_fields_labels::choice_label;
use crate::model::rules_values::parse_list;
use crate::ui::controls::tick;

/// Champ texte : chaque frappe met à jour le brouillon.
pub fn text_field(ed: &Rc<Editor>, path: Vec<Seg>, value: &str) -> Entry {
    let entry = field(value);
    entry.set_max_length(1000);
    entry.set_hexpand(true);
    let weak = Rc::downgrade(ed);
    entry.connect_changed(move |e| {
        if let Some(ed) = weak.upgrade() {
            ed.set(&path, json!(e.text().to_string()));
        }
    });
    entry
}

/// Champ numérique : entier si possible, sinon décimal ; une saisie illisible laisse la valeur précédente.
pub fn number_field(ed: &Rc<Editor>, path: Vec<Seg>, value: &Value) -> Entry {
    let entry = field(&value.as_f64().map_or(String::new(), |n| if n.fract() == 0.0 { format!("{}", n as i64) } else { n.to_string() }));
    entry.set_width_chars(10);
    let weak = Rc::downgrade(ed);
    entry.connect_changed(move |e| {
        let text = e.text().replace(',', ".");
        if text.trim().is_empty() {
            if let Some(ed) = weak.upgrade() {
                ed.set(&path, Value::Null);
            }
            return;
        }
        let parsed = text.parse::<i64>().map(|n| json!(n)).or_else(|_| text.parse::<f64>().map(|n| json!(n)));
        if let (Ok(number), Some(ed)) = (parsed, weak.upgrade()) {
            ed.set(&path, number);
        }
    });
    entry
}

/// Case à cocher liée à un booléen.
pub fn bool_field(ed: &Rc<Editor>, path: Vec<Seg>, on: bool) -> Button {
    let button = tick(on);
    let weak = Rc::downgrade(ed);
    button.connect_clicked(move |b| {
        let now = !crate::ui::controls::is_ticked(b);
        crate::ui::controls::set_tick(b, now);
        if let Some(ed) = weak.upgrade() {
            ed.set(&path, json!(now));
        }
    });
    button
}

/// Choix parmi des valeurs fixes, affichées avec leur libellé français.
pub fn choice_field(ed: &Rc<Editor>, path: Vec<Seg>, choices: &[String], current: &str) -> Button {
    let options: Vec<Choice> = choices.iter().map(|c| (c.clone(), choice_label(c))).collect();
    select_button(ed, current, options, move |ed, value| ed.set(&path, json!(value)))
}

/// Liste de textes saisie « a, b, c ».
pub fn list_field(ed: &Rc<Editor>, path: Vec<Seg>, items: &[Value]) -> Entry {
    let joined = items.iter().filter_map(Value::as_str).collect::<Vec<_>>().join(", ");
    let entry = field(&joined);
    entry.set_hexpand(true);
    let weak = Rc::downgrade(ed);
    entry.connect_changed(move |e| {
        if let Some(ed) = weak.upgrade() {
            ed.set(&path, json!(parse_list(&e.text())));
        }
    });
    entry
}

/// Liste de nombres saisie « 1, 5, 12 » ; les éléments illisibles sont ignorés.
pub fn numbers_field(ed: &Rc<Editor>, path: Vec<Seg>, items: &[Value]) -> Entry {
    let joined = items.iter().filter_map(Value::as_i64).map(|n| n.to_string()).collect::<Vec<_>>().join(", ");
    let entry = field(&joined);
    entry.set_hexpand(true);
    let weak = Rc::downgrade(ed);
    entry.connect_changed(move |e| {
        let numbers: Vec<i64> = parse_list(&e.text()).iter().filter_map(|t| t.parse().ok()).collect();
        if let Some(ed) = weak.upgrade() {
            ed.set(&path, json!(numbers));
        }
    });
    entry
}
