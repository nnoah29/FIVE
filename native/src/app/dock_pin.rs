//! Barre du bas : épingler ou retirer un dossier ou un fichier, entrée du menu contextuel.

use std::rc::Rc;

use five_engine::entry::Entry;

use super::popover::MenuEntry;
use super::shared::App;
use crate::model::dock::{pin_dock, unpin_dock};

impl App {
    /// Chemin absolu d'un élément affiché, tel qu'enregistré dans la barre du bas.
    fn dock_key(&self, item: &Entry) -> Option<String> {
        self.root.resolve(&item.path).ok().map(|full| full.to_string_lossy().into_owned())
    }

    /// Épingle un dossier ou un fichier dans la barre du bas.
    pub fn pin_to_dock(self: &Rc<Self>, item: &Entry) {
        let Some(key) = self.dock_key(item) else { return };
        let mut added = false;
        self.change_setting(|s| added = pin_dock(&mut s.dock, &key));
        self.toast(if added { "Épinglé dans la barre du bas" } else { "Déjà dans la barre du bas" });
    }

    /// Retire un raccourci de la barre du bas.
    pub fn unpin_from_dock(self: &Rc<Self>, key: &str) {
        self.change_setting(|s| {
            let _ = unpin_dock(&mut s.dock, key);
        });
        self.toast("Retiré de la barre du bas");
    }

    /// Entrée du menu d'un élément : épingler dans la barre du bas, ou l'en retirer s'il y est déjà.
    pub fn dock_toggle_entry(&self, item: &Entry) -> MenuEntry {
        let (target, key) = (item.clone(), self.dock_key(item).unwrap_or_default());
        if self.set.borrow().dock.contains(&key) {
            return MenuEntry::new("x", "Retirer de la barre du bas", move |app| app.unpin_from_dock(&key));
        }
        MenuEntry::new("favorites", "Épingler dans la barre du bas", move |app| app.pin_to_dock(&target))
    }
}
