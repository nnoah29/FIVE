//! Une ligne de dossier surveillé : dossier, sous-dossiers, éléments cachés, profondeur.

use std::rc::Rc;

use gtk4::Box as GtkBox;
use gtk4::prelude::*;
use serde_json::{Value, json};

use super::fields_simple::{bool_field, number_field};
use super::fields_special::folder_field;
use super::state::Editor;
use crate::model::rules_draft::{Seg, at_mut, key, remove_item};
use crate::ui::buttons::icon_button;
use crate::ui::widgets::{hbox, label, vbox};

/// Ligne d'un dossier surveillé ; le bouton retire la ligne.
pub fn watch_row(ed: &Rc<Editor>, index: usize, watch: &Value) -> GtkBox {
    let block = vbox(8);
    block.add_css_class("rule-item");
    let base = vec![key("watch"), Seg::Idx(index)];
    let at = |name: &str| [base.clone(), vec![key(name)]].concat();
    let top = hbox(8);
    let folder = folder_field(ed, at("path"), watch["path"].as_str().unwrap_or(""));
    folder.set_hexpand(true);
    top.append(&folder);
    let remove = icon_button("x", &["small"]);
    let weak = Rc::downgrade(ed);
    remove.connect_clicked(move |_| {
        if let Some(ed) = weak.upgrade() {
            ed.mutate(|d| at_mut(d, &[key("watch")]).iter_mut().for_each(|list| remove_item(list, index)));
        }
    });
    top.append(&remove);
    block.append(&top);
    let options = hbox(12);
    options.append(&label("SOUS-DOSSIERS", &["rule-field-label"]));
    options.append(&bool_field(ed, at("recursive"), watch["recursive"] == true));
    options.append(&label("IGNORER LES CACHÉS", &["rule-field-label"]));
    options.append(&bool_field(ed, at("ignore_hidden"), watch["ignore_hidden"] != false));
    options.append(&label("PROFONDEUR MAX", &["rule-field-label"]));
    options.append(&number_field(ed, at("max_depth"), &json!(watch["max_depth"])));
    block.append(&options);
    block
}
