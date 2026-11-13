//! Navigation : chargement d'un emplacement, dossier parent, vues virtuelles.

use std::rc::Rc;

use super::data::{Listing, read_location};
use super::shared::App;
use super::work::spawn_work;

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
}
