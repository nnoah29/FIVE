//! Sections de l'éditeur : nom, dossiers surveillés, déclencheurs, conditions (OU de blocs ET), actions.

use std::rc::Rc;

use gtk4::Box as GtkBox;
use gtk4::prelude::*;
use serde_json::json;

use super::conditions::{conditions_list, section_box};
use super::items::typed_item;
use super::place::Place;
use super::sections_basic::{add_button, header_section, watch_section};
use super::state::Editor;
use crate::model::rules_draft::{Seg, at_mut, key, remove_item};
use crate::ui::buttons::icon_button;
use crate::ui::widgets::{hbox, label};

/// Déclencheurs : la règle agit dès que l'un d'eux se produit.
fn triggers_section(ed: &Rc<Editor>) -> GtkBox {
    let block = section_box("Quand");
    let total = ed.get(&[key("triggers")]).as_array().map_or(0, Vec::len);
    (0..total).for_each(|index| block.append(&typed_item(ed, &Place { list: "triggers", container: vec![key("triggers")], index, total })));
    block.append(&add_button(ed, "Ajouter un déclencheur", "triggers", |ed| ed.catalog.blank("triggers", "created")));
    block
}

/// Conditions : des blocs « ET » reliés par « OU » ; la règle agit si un bloc est entièrement vrai.
fn alternatives_section(ed: &Rc<Editor>) -> GtkBox {
    let block = section_box("Si");
    let total = ed.get(&[key("alternatives")]).as_array().map_or(0, Vec::len);
    for index in 0..total {
        let head = hbox(8);
        let title = label(if index == 0 { "TOUTES CES CONDITIONS" } else { "OU TOUTES CELLES-CI" }, &["rule-field-label"]);
        title.set_hexpand(true);
        head.append(&title);
        let remove = icon_button("x", &["small"]);
        remove.set_sensitive(total > 1);
        let weak = Rc::downgrade(ed);
        remove.connect_clicked(move |_| {
            if let Some(ed) = weak.upgrade() {
                ed.mutate(|d| at_mut(d, &[key("alternatives")]).iter_mut().for_each(|list| remove_item(list, index)));
            }
        });
        head.append(&remove);
        block.append(&head);
        block.append(&conditions_list(ed, vec![key("alternatives"), Seg::Idx(index), key("conditions")]));
    }
    block.append(&add_button(ed, "Ajouter une alternative (OU)", "alternatives", |_| json!({"conditions": []})));
    block
}

/// Actions, appliquées dans l'ordre.
fn actions_section(ed: &Rc<Editor>) -> GtkBox {
    let block = section_box("Alors");
    let total = ed.get(&[key("actions")]).as_array().map_or(0, Vec::len);
    (0..total).for_each(|index| block.append(&typed_item(ed, &Place { list: "actions", container: vec![key("actions")], index, total })));
    block.append(&add_button(ed, "Ajouter une action", "actions", |ed| ed.catalog.blank("actions", "move_to")));
    block
}

/// Remplit le corps de l'éditeur : toutes les sections.
pub fn build_body(ed: &Rc<Editor>) {
    for section in [header_section(ed), watch_section(ed), triggers_section(ed), alternatives_section(ed), actions_section(ed)] {
        ed.body.append(&section);
    }
}
