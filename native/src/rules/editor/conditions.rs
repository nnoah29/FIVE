//! Liste de conditions (une alternative, ou les tests imbriqués d'une composition) et bloc de section.

use std::rc::Rc;

use gtk4::Box as GtkBox;
use gtk4::prelude::*;
use serde_json::json;

use super::items::typed_item;
use super::place::Place;
use super::state::Editor;
use crate::model::rules_draft::{Seg, at_mut};
use crate::ui::widgets::{label, text_button, vbox};

/// Bloc de section : titre en petites capitales et, dessous, ses éléments.
pub fn section_box(title: &str) -> GtkBox {
    let block = vbox(10);
    block.add_css_class("rule-block");
    block.append(&label(&title.to_uppercase(), &["kinds-title"]));
    block
}

/// Conditions du tableau `container`, chacune éditable, suivies d'un bouton d'ajout.
pub fn conditions_list(ed: &Rc<Editor>, container: Vec<Seg>) -> GtkBox {
    let list = vbox(8);
    let items = ed.get(&container).as_array().cloned().unwrap_or_default();
    for index in 0..items.len() {
        list.append(&typed_item(ed, &Place { list: "conditions", container: container.clone(), index, total: items.len() }));
    }
    let add = text_button("Ajouter une condition", Some("plus"), &[]);
    let (weak, target) = (Rc::downgrade(ed), container);
    add.connect_clicked(move |_| {
        let Some(ed) = weak.upgrade() else { return };
        let fresh = json!({"negated": false, "test": ed.catalog.blank("conditions", "name")});
        ed.mutate(|d| {
            if let Some(items) = at_mut(d, &target).and_then(|v| v.as_array_mut()) {
                items.push(fresh);
            }
        });
    });
    list.append(&add);
    list
}
