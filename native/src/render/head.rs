//! Rendu de l'en-tête : titre, fil d'Ariane déroulant, filtres, utilisateur.

use std::rc::Rc;

use gtk4::Button;
use gtk4::prelude::*;

use crate::app::shared::{App, on_click};
use crate::model::kinds::FILTERS;
use crate::ui::classes::{clear, set_class};
use crate::ui::widgets::label;

/// Bouton de texte brut du fil d'Ariane.
fn crumb(text: &str, class: &str) -> Button {
    let button = Button::new();
    button.set_child(Some(&label(text, &[])));
    button.add_css_class(class);
    button
}

impl App {
    /// Séparateur du fil d'Ariane : ouvre la liste des sous-dossiers de `parent`.
    fn crumb_sep(self: &Rc<Self>, parent: String) -> Button {
        let sep = crumb("/", "crumb-sep");
        on_click(&sep, self, move |app| app.crumb_menu(parent.clone()));
        sep
    }

    /// Reconstruit le fil d'Ariane : racine, séparateurs déroulants et dossiers.
    fn render_crumbs(self: &Rc<Self>) {
        clear(&self.ui.folders.crumbs);
        let (path, root, view) = {
            let st = self.st.borrow();
            (st.path.clone(), st.root.clone(), st.virtual_view.clone())
        };
        let home = crumb(if view.is_some() { "Vue virtuelle" } else { &root }, "crumb");
        on_click(&home, self, |app| app.go(""));
        self.ui.folders.crumbs.append(&home);
        if view.is_some() {
            return;
        }
        self.ui.folders.crumbs.append(&self.crumb_sep(String::new()));
        let parts: Vec<&str> = path.split('/').filter(|p| !p.is_empty()).collect();
        for (at, part) in parts.iter().enumerate() {
            let upto = parts[..=at].join("/");
            let button = crumb(part, "crumb");
            let target = upto.clone();
            on_click(&button, self, move |app| app.go(&target));
            set_class(&button, "last", at + 1 == parts.len());
            self.ui.folders.crumbs.append(&button);
            self.ui.folders.crumbs.append(&self.crumb_sep(upto));
        }
    }

    /// Reconstruit les pastilles de filtre par type.
    fn render_pills(self: &Rc<Self>) {
        clear(&self.ui.files.pills);
        let current = self.st.borrow().filter.clone();
        for (id, text) in FILTERS {
            let pill = Button::new();
            pill.set_child(Some(&label(text, &[])));
            pill.add_css_class("pill");
            set_class(&pill, "pressed", current == id);
            on_click(&pill, self, move |app| app.toggle_filter(id));
            self.ui.files.pills.append(&pill);
        }
    }

    /// Réécrit titres, fil d'Ariane, filtres et pastille de l'utilisateur.
    pub fn render_head(self: &Rc<Self>) {
        self.ui.folders.title.set_text(&self.page_title().to_uppercase());
        let (virt, searching, user, root) = {
            let st = self.st.borrow();
            (st.virtual_view.is_some(), st.searching(), st.user.clone(), st.root.clone())
        };
        self.ui.files.title.set_text(
            &(if virt {
                "Éléments"
            } else if searching {
                "Résultats"
            } else {
                "Tous les fichiers"
            })
            .to_uppercase(),
        );
        self.render_crumbs();
        self.render_pills();
        self.ui.top.user_name.set_text(&user);
        self.ui.top.user_root.set_text(&root.to_uppercase());
        self.ui.top.avatar.set_text(&user.chars().next().unwrap_or('?').to_uppercase().to_string());
    }
}
