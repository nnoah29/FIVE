//! Aller quelque part : dossier, vue virtuelle, parent, effacement des filtres.

use std::rc::Rc;

use gtk4::prelude::*;

use super::shared::App;
use crate::model::paths::parent_of;

impl App {
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
