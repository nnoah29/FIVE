//! Un élément typé (déclencheur, condition ou action) : famille, type, ordre, retrait, et les champs de son type.

use std::rc::Rc;

use gtk4::Box as GtkBox;
use gtk4::prelude::*;
use serde_json::Value;

use super::field::field_row;
use super::fields_simple::bool_field;
use super::place::{Place, kind_label};
use super::select::{Choice, select_button};
use super::state::Editor;
use crate::model::rules_draft::{Seg, at_mut, key, move_item, remove_item};
use crate::ui::buttons::icon_button;
use crate::ui::widgets::{hbox, label, vbox};

/// Monter, descendre et retirer l'élément.
fn order_buttons(ed: &Rc<Editor>, place: &Place) -> GtkBox {
    let row = hbox(4);
    for (glyph, delta) in [("up", -1isize), ("down", 1)] {
        let button = icon_button(glyph, &["small"]);
        button.set_sensitive(if delta < 0 { place.index > 0 } else { place.index + 1 < place.total });
        let (weak, container, index) = (Rc::downgrade(ed), place.container.clone(), place.index);
        button.connect_clicked(move |_| {
            if let Some(ed) = weak.upgrade() {
                ed.mutate(|d| at_mut(d, &container).iter_mut().for_each(|list| move_item(list, index, delta)));
            }
        });
        row.append(&button);
    }
    let remove = icon_button("x", &["small"]);
    let (weak, container, index) = (Rc::downgrade(ed), place.container.clone(), place.index);
    remove.connect_clicked(move |_| {
        if let Some(ed) = weak.upgrade() {
            ed.mutate(|d| at_mut(d, &container).iter_mut().for_each(|list| remove_item(list, index)));
        }
    });
    row.append(&remove);
    row
}

/// Remplace l'élément par un élément neuf du type choisi.
fn retype(ed: &Rc<Editor>, list: &'static str, body: &[Seg], kind: &str) {
    let blank = ed.catalog.blank(list, kind);
    let body = body.to_vec();
    ed.mutate(|d| {
        if let Some(slot) = at_mut(d, &body) {
            *slot = blank;
        }
    });
}

/// Tête : « NON » pour une condition, famille et type, ordre.
fn item_head(ed: &Rc<Editor>, place: &Place, kind: &str) -> GtkBox {
    let head = hbox(8);
    if place.list == "conditions" {
        head.append(&label("NON", &["mono", "dim"]));
        head.append(&bool_field(
            ed,
            [place.item(), vec![key("negated")]].concat(),
            ed.get(&[place.item(), vec![key("negated")]].concat()) == Value::Bool(true),
        ));
    }
    let (list, body) = (place.list, place.body());
    let family = ed.catalog.family_of(list, kind);
    if place.list != "triggers" {
        let families: Vec<Choice> = ed.catalog.families(list).into_iter().map(|f| (f.clone(), f)).collect();
        let first = body.clone();
        head.append(&select_button(ed, &family, families, move |ed, picked| {
            if let Some(first_kind) = ed.catalog.kinds_of(list, &picked).first() {
                retype(ed, list, &first, first_kind);
            }
        }));
    }
    let kinds: Vec<Choice> = ed.catalog.kinds_of(list, &family).into_iter().map(|k| (k.clone(), kind_label(list, &k))).collect();
    let target = body;
    let picker = select_button(ed, kind, kinds, move |ed, picked| retype(ed, list, &target, &picked));
    picker.set_hexpand(true);
    head.append(&picker);
    head.append(&order_buttons(ed, place));
    head
}

/// Élément complet : tête et champs du type courant.
pub fn typed_item(ed: &Rc<Editor>, place: &Place) -> GtkBox {
    let block = vbox(8);
    block.add_css_class("rule-item");
    let body = ed.get(&place.body());
    let kind = body["type"].as_str().unwrap_or("").to_string();
    block.append(&item_head(ed, place, &kind));
    let fields = ed.catalog.entry(place.list, &kind).and_then(|e| e["fields"].as_array().cloned()).unwrap_or_default();
    fields.iter().for_each(|descriptor| block.append(&field_row(ed, &place.body(), descriptor, &body)));
    block
}
