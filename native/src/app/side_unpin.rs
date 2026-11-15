//! Retrait d'un dossier épinglé de la barre latérale : clic droit sur l'entrée ou menu du dossier.

use std::path::Path;
use std::rc::Rc;

use five_engine::realpath::real_path;
use gtk4::prelude::*;
use gtk4::{Button, GestureClick};

use super::menu_place::MenuPlace;
use super::popover::MenuEntry;
use super::shared::App;

impl App {
    /// Vrai si le dossier (chemin relatif à la racine) figure dans la barre latérale.
    pub fn is_side_pinned(&self, path: &str) -> bool {
        self.side_places().iter().any(|p| p.path == path)
    }

    /// Entrée du menu d'un dossier : épingler, ou retirer s'il est déjà épinglé.
    pub fn pin_toggle_entry(&self, target: five_engine::entry::Entry) -> MenuEntry {
        if self.is_side_pinned(&target.path) {
            let path = target.path.clone();
            return MenuEntry::new("x", "Retirer de la barre latérale", move |app| app.unpin_side_folder(&path));
        }
        MenuEntry::new("favorites", "Épingler dans la barre latérale", move |app| app.pin_entry(&target))
    }

    /// Retire de la barre latérale le dossier de `path` (relatif à la racine).
    pub fn unpin_side_folder(self: &Rc<Self>, path: &str) {
        let same = |f: &crate::model::side_folders::SideFolder| self.root.relative(&real_path(Path::new(&f.path))) == path;
        self.edit_side_folders(|list| list.retain(|f| !same(f)));
        self.toast("Dossier retiré de la barre latérale");
    }
}

/// Clic droit sur une entrée épinglée de la barre : propose de la retirer.
pub fn attach_unpin_menu(app: &Rc<App>, button: &Button, path: &str) {
    let click = GestureClick::new();
    click.set_button(3);
    let (weak, path, anchor) = (Rc::downgrade(app), path.to_string(), button.clone());
    click.connect_pressed(move |_, _, x, y| {
        let Some(app) = weak.upgrade() else { return };
        let target = path.clone();
        let entry = MenuEntry::new("x", "Retirer de la barre latérale", move |app| app.unpin_side_folder(&target));
        app.popover_at(&anchor, MenuPlace::Pointer(x, y), vec![entry]);
    });
    button.add_controller(click);
}
