//! Menu d'actions d'un élément (bouton « ⋮ » ou clic droit).

use std::rc::Rc;

use five_engine::entry::Entry;
use gtk4::Widget;
use gtk4::prelude::*;

use super::popover::MenuEntry;
use super::shared::App;
use crate::model::paths::in_trash;

impl App {
    /// Éléments visés par une action de menu : toute la sélection si l'élément en fait partie, sinon lui seul.
    fn menu_targets(&self, item: &Entry) -> Vec<Entry> {
        let st = self.st.borrow();
        if st.picked.contains(&item.path) && st.picked.len() > 1 { st.picked_items() } else { vec![item.clone()] }
    }

    /// Entrées du menu d'un ensemble d'éléments.
    fn menu_entries(&self, item: &Entry, targets: Vec<Entry>) -> Vec<MenuEntry> {
        let single = targets.len() == 1;
        let definitive = targets.iter().all(|t| in_trash(&t.path));
        let mut out = Vec::new();
        let it = item.clone();
        if single {
            let open_label = if item.is_dir { "Ouvrir le dossier" } else { "Ouvrir" };
            let target = it.clone();
            out.push(MenuEntry::new(if item.is_dir { "open" } else { "file" }, open_label, move |app| app.open_item(&target)));
        }
        if single && !item.is_dir {
            let (save, look) = (it.clone(), it.clone());
            out.push(MenuEntry::new("down", "Enregistrer une copie…", move |app| app.save_copy(&save)));
            out.push(MenuEntry::new("eye", "Aperçu rapide", move |app| app.quick_look(Some(look.clone()))).hint("Espace"));
        }
        if single {
            let target = it.clone();
            out.push(MenuEntry::new("copy", "Copier le chemin", move |app| app.copy_path(&target)));
        }
        let (copy, cut, rename, delete) = (targets.clone(), targets.clone(), targets.clone(), targets);
        out.push(MenuEntry::new("copy", "Copier", move |app| app.set_clip("copy", &copy)).hint("Ctrl C"));
        out.push(MenuEntry::new("cut", "Couper", move |app| app.set_clip("move", &cut)).hint("Ctrl X"));
        out.push(MenuEntry::new("pen", "Renommer", move |app| app.rename_items(rename.clone())).hint("F2"));
        out.push(MenuEntry::new("trash2", if definitive { "Supprimer définitivement" } else { "Mettre à la corbeille" }, move |app| {
            app.delete_items(delete.clone())
        }));
        out
    }

    /// Ouvre le menu d'un élément sous `anchor` ; sélectionne d'abord l'élément s'il ne l'est pas.
    pub fn open_menu_at(self: &Rc<Self>, anchor: &impl IsA<Widget>, item: &Entry, index: Option<usize>) {
        if let Some(i) = index.filter(|_| !self.st.borrow().picked.contains(&item.path)) {
            self.select(i as isize, gtk4::gdk::ModifierType::empty());
        }
        let targets = self.menu_targets(item);
        let entries = self.menu_entries(item, targets);
        self.popover(anchor, entries);
    }
}
