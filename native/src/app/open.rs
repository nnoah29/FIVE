//! Sélection globale, déplacement au clavier et ouverture des éléments.

use std::rc::Rc;

use five_engine::entry::Entry;
use gtk4::gdk::ModifierType;
use gtk4::prelude::FileExt;

use super::shared::App;

impl App {
    /// Sélectionne tous les éléments affichés.
    pub fn select_all(self: &Rc<Self>) {
        {
            let mut st = self.st.borrow_mut();
            st.picked = st.shown.iter().map(|i| i.path.clone()).collect();
            st.sel = if st.shown.is_empty() { -1 } else { 0 };
        }
        self.paint_selection();
        self.show_preview();
    }

    /// Déplace la sélection de `delta` lignes.
    pub fn move_sel(self: &Rc<Self>, delta: isize) {
        let (count, current) = {
            let st = self.st.borrow();
            (st.shown.len() as isize, st.sel)
        };
        if count > 0 {
            self.select((current + delta).clamp(0, count - 1), ModifierType::empty());
        }
    }

    /// Ouvre l'élément affiché à la position `index`.
    pub fn open_index(self: &Rc<Self>, index: usize) {
        let item = self.st.borrow().shown.get(index).cloned();
        if let Some(item) = item {
            self.open_item(&item);
        }
    }

    /// Ouvre un dossier dans FIVE, un fichier avec l'application par défaut du système.
    pub fn open_item(self: &Rc<Self>, item: &Entry) {
        if item.is_dir {
            self.reset_search();
            self.go(&item.path);
        } else if let Ok(full) = self.root.resolve(&item.path) {
            let uri = gtk4::gio::File::for_path(full).uri();
            if gtk4::gio::AppInfo::launch_default_for_uri(&uri, None::<&gtk4::gio::AppLaunchContext>).is_err() {
                self.toast("Aucune application pour ouvrir ce fichier");
            }
        }
    }
}
