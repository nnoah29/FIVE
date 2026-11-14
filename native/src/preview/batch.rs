//! Panneau d'une sélection multiple : décompte et actions groupées.

use std::rc::Rc;

use five_engine::entry::Entry;
use gtk4::Widget;
use gtk4::prelude::*;

use super::facts::Facts;
use crate::app::shared::{App, on_click};
use crate::model::format::format_size;
use crate::model::paths::in_trash;
use crate::ui::widgets::{label, text_button};

impl App {
    /// Boutons d'actions groupées : renommer, copier, couper, compresser, supprimer.
    fn batch_buttons(self: &Rc<Self>, items: &[Entry], definitive: bool) -> gtk4::FlowBox {
        let flow = gtk4::FlowBox::new();
        flow.set_selection_mode(gtk4::SelectionMode::None);
        flow.set_column_spacing(8);
        flow.set_row_spacing(8);
        let buttons = [
            ("Renommer", "pen", "go"),
            ("Copier", "copy", ""),
            ("Couper", "cut", ""),
            ("Compresser en .zip", "archive", ""),
            (if definitive { "Supprimer définitivement" } else { "Mettre à la corbeille" }, "trash2", ""),
        ];
        for (at, (text, icon, class)) in buttons.iter().enumerate() {
            let classes: &[&str] = if class.is_empty() { &[] } else { std::slice::from_ref(class) };
            let button = text_button(text, Some(icon), classes);
            let list = items.to_vec();
            on_click(&button, self, move |app| match at {
                0 => app.rename_items(list.clone()),
                1 => app.set_clip("copy", &list),
                2 => app.set_clip("move", &list),
                3 => app.archive_items("zip", &list),
                _ => app.delete_items(list.clone()),
            });
            flow.append(&button);
        }
        flow
    }

    /// Panneau d'actions pour plusieurs éléments sélectionnés.
    pub fn batch_panel(self: &Rc<Self>, items: &[Entry]) -> Vec<Widget> {
        let files: Vec<&Entry> = items.iter().filter(|i| !i.is_dir).collect();
        let facts = Facts::new();
        facts.add("Sélection", &format!("{} éléments", items.len()));
        facts.add("Fichiers", &files.len().to_string());
        facts.add("Dossiers", &(items.len() - files.len()).to_string());
        facts.add("Taille", &format_size(files.iter().map(|i| i.size).sum(), self.set.borrow().binary_units));
        let definitive = items.iter().all(|i| in_trash(&i.path));
        let flow = self.batch_buttons(items, definitive);
        vec![label(&format!("{} ÉLÉMENTS", items.len()), &["h2"]).upcast(), facts.grid.clone().upcast(), flow.upcast()]
    }
}
