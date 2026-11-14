//! Choix multiples : une rangée de boutons à bascule (jours de la semaine, familles de fichiers, états Git).

use std::rc::Rc;

use gtk4::Box as GtkBox;
use gtk4::prelude::*;
use serde_json::{Value, json};

use super::state::Editor;
use crate::model::rules_draft::Seg;
use crate::model::rules_fields_labels::choice_label;
use crate::ui::classes::set_class;
use crate::ui::widgets::{hbox, label};

/// Boutons à bascule pour des valeurs fixes ; le brouillon reçoit la liste des valeurs pressées, dans l'ordre du catalogue.
pub fn multi_field(ed: &Rc<Editor>, path: Vec<Seg>, choices: &[String], current: &[Value]) -> GtkBox {
    let row = hbox(0);
    row.add_css_class("seg");
    let selected: Rc<std::cell::RefCell<Vec<String>>> = Rc::new(std::cell::RefCell::new(current.iter().filter_map(|v| v.as_str().map(String::from)).collect()));
    for choice in choices {
        let button = gtk4::Button::new();
        button.set_child(Some(&label(&choice_label(choice).to_uppercase(), &[])));
        button.add_css_class("seg-btn");
        set_class(&button, "pressed", selected.borrow().contains(choice));
        let (weak, picked, path, all, own) = (Rc::downgrade(ed), selected.clone(), path.clone(), choices.to_vec(), choice.clone());
        button.connect_clicked(move |b| {
            let mut list = picked.borrow_mut();
            let now = !list.contains(&own);
            if now {
                list.push(own.clone())
            } else {
                list.retain(|v| *v != own)
            }
            set_class(b, "pressed", now);
            let ordered: Vec<&String> = all.iter().filter(|v| list.contains(v)).collect();
            if let Some(ed) = weak.upgrade() {
                ed.set(&path, json!(ordered));
            }
        });
        row.append(&button);
    }
    row
}
