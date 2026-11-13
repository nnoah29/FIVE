//! Filtre par type, déroulant du fil d'Ariane et enregistrement d'une copie.

use std::rc::Rc;

use gtk4::gio;
use gtk4::prelude::*;

use super::popover::MenuEntry;
use super::shared::App;
use super::work::spawn_work;
use crate::model::sorting::natural_cmp;

impl App {
    /// Applique le filtre `id` ; le choisir une seconde fois revient à « Tout ».
    pub fn toggle_filter(self: &Rc<Self>, id: &str) {
        {
            let mut st = self.st.borrow_mut();
            st.filter = if id == "all" || st.filter == id { "all".to_string() } else { id.to_string() };
            st.sel = -1;
        }
        self.render(None);
        self.show_preview();
    }

    /// Ouvre la liste des sous-dossiers de `parent`, ancrée au séparateur correspondant.
    pub fn crumb_menu(self: &Rc<Self>, parent: String) {
        let (root, hidden, app) = (self.root.clone(), self.st.borrow().hidden, self.clone());
        let anchor = self.ui.folders.crumbs.last_child();
        spawn_work(
            move || {
                let full = root.resolve(&parent).map_err(|r| r.message)?;
                five_engine::listing::list_dir(&root, &full, hidden).map_err(|e| five_engine::refus::Refus::from(e).message)
            },
            move |result| match result {
                Ok(items) => {
                    let mut dirs: Vec<_> = items.into_iter().filter(|i| i.is_dir).collect();
                    dirs.sort_by(|a, b| natural_cmp(&a.name, &b.name));
                    let mut entries: Vec<MenuEntry> =
                        dirs.into_iter().take(40).map(|d| MenuEntry::new("folder", &d.name.clone(), move |app| app.go(&d.path))).collect();
                    if entries.is_empty() {
                        entries.push(MenuEntry::new("folder", "Aucun sous-dossier", |_| {}));
                    }
                    if let Some(widget) = anchor.clone() {
                        app.popover(&widget, entries);
                    }
                }
                Err(message) => app.toast(&message),
            },
        );
    }

    /// Propose d'enregistrer une copie du fichier ailleurs, avec la boîte de dialogue du système.
    pub fn save_copy(self: &Rc<Self>, item: &five_engine::entry::Entry) {
        let Ok(source) = self.root.resolve(&item.path) else { return };
        let dialog = gtk4::FileDialog::new();
        dialog.set_initial_name(Some(&item.name));
        let app = self.clone();
        dialog.save(Some(&self.ui.window), None::<&gio::Cancellable>, move |result| {
            let Ok(target) = result else { return };
            let Some(destination) = target.path() else { return };
            let again = app.clone();
            spawn_work(
                move || std::fs::copy(&source, &destination).map(|_| ()).map_err(|e| five_engine::refus::Refus::from(e).message),
                move |outcome| match outcome {
                    Ok(()) => again.toast("Copie enregistrée"),
                    Err(message) => again.toast(&message),
                },
            );
        });
    }
}
