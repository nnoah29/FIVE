//! Navigation : chargement d'un emplacement, dossier parent, vues virtuelles.

use std::rc::Rc;

use gtk4::prelude::*;

use super::data::{read_location, Listing};
use super::shared::App;
use super::work::spawn_work;
use crate::model::paths::parent_of;

impl App {
    /// Lit un emplacement en arrière-plan puis met l'interface à jour ; les lectures dépassées sont ignorées.
    pub fn load(self: &Rc<Self>, path: String, view: Option<String>) {
        let ticket = {
            let mut st = self.st.borrow_mut();
            st.ticket += 1;
            st.ticket
        };
        let (query, hidden) = {
            let st = self.st.borrow();
            (st.query.clone(), st.hidden)
        };
        let (root, app, wanted) = (self.root.clone(), self.clone(), view.clone());
        spawn_work(move || read_location(&root, &path, wanted.as_deref(), &query, hidden), move |result| app.finish_load(ticket, view, result));
    }

    /// Applique le résultat d'une lecture si elle est toujours la plus récente.
    fn finish_load(self: &Rc<Self>, ticket: u64, view: Option<String>, result: Result<Listing, String>) {
        if self.st.borrow().ticket != ticket {
            return;
        }
        match result {
            Ok(listing) => {
                {
                    let mut st = self.st.borrow_mut();
                    (st.path, st.items, st.free, st.total) = (listing.path, listing.items, listing.free, listing.total);
                    (st.sel, st.virtual_view) = (-1, view);
                    st.picked.clear();
                }
                self.render(None);
                self.show_preview();
                self.after_load();
            }
            Err(message) => {
                self.st.borrow_mut().items.clear();
                self.render(Some(&message));
                self.toast(&message);
            }
        }
    }

    /// Relit l'emplacement courant.
    pub fn reload(self: &Rc<Self>) {
        let (path, view) = {
            let st = self.st.borrow();
            (st.path.clone(), st.virtual_view.clone())
        };
        self.load(path, view);
    }

    /// Ouvre un dossier de la racine.
    pub fn go(self: &Rc<Self>, path: &str) {
        self.close_drawers();
        self.load(path.to_string(), None);
    }

    /// Ouvre une vue virtuelle (récents, favoris, réseau) en effaçant la recherche.
    pub fn go_view(self: &Rc<Self>, view: &str) {
        self.reset_search();
        self.close_drawers();
        self.load(String::new(), Some(view.to_string()));
    }

    /// Remonte au dossier parent en effaçant la recherche.
    pub fn go_up(self: &Rc<Self>) {
        self.reset_search();
        let parent = parent_of(&self.st.borrow().path);
        self.go(&parent);
    }

    /// Efface la requête de recherche et le champ.
    pub fn reset_search(self: &Rc<Self>) {
        self.st.borrow_mut().query.clear();
        self.ui.top.search.set_text("");
    }

    /// Efface les filtres et la recherche, puis relit le dossier.
    pub fn clear_filters(self: &Rc<Self>) {
        self.st.borrow_mut().filter = "all".to_string();
        self.reset_search();
        self.reload();
    }
}
