//! Lignes et groupes de la fenêtre des doublons.

use std::rc::Rc;

use gtk4::prelude::*;
use serde_json::Value;

use super::dups::Marked;
use crate::app::shared::App;
use crate::model::format::format_size;
use crate::ui::controls::{set_tick, tick};
use crate::ui::widgets::{hbox, label, vbox};

impl App {
    /// Une case par fichier d'un groupe : cocher marque la copie pour la corbeille.
    pub fn dup_row(self: &Rc<Self>, file: &str, marked: &Marked, refresh: Rc<dyn Fn()>) -> gtk4::Box {
        let row = hbox(8);
        let check = tick(marked.borrow().contains(file));
        let (path, marked) = (file.to_string(), marked.clone());
        check.connect_clicked(move |button| {
            if !marked.borrow_mut().remove(&path) {
                marked.borrow_mut().insert(path.clone());
            }
            set_tick(button, marked.borrow().contains(&path));
            refresh();
        });
        row.append(&check);
        row.append(&label(&format!("/{file}"), &["mono-path"]));
        row
    }

    /// Bloc d'un groupe de copies identiques.
    pub fn dup_group(self: &Rc<Self>, group: &Value, marked: &Marked, refresh: &Rc<dyn Fn()>) -> gtk4::Box {
        let files: Vec<String> = group["files"].as_array().cloned().unwrap_or_default().iter().filter_map(|f| f.as_str().map(String::from)).collect();
        let size = group["size"].as_u64().unwrap_or(0);
        let block = vbox(0);
        block.add_css_class("dup");
        block.append(&label(
            &format!("{} COPIES · {} CHACUNE", files.len(), format_size(size, self.set.borrow().binary_units)).to_uppercase(),
            &["kinds-title"],
        ));
        files.iter().for_each(|f| block.append(&self.dup_row(f, marked, refresh.clone())));
        block
    }
}
