//! Rendu de l'en-tête de colonnes du tableau.

use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{Align, Button};

use crate::app::shared::{App, on_click};
use crate::model::sorting::SortKey;
use crate::ui::classes::{clear, fixed, set_class};
use crate::ui::icon::icon_with_stroke;
use crate::ui::widgets::{hbox, label};

/// Largeurs des colonnes Type, Taille et Modifié.
pub fn column_widths(full_dates: bool) -> (i32, i32, i32) {
    (128, 84, if full_dates { 150 } else { 104 })
}

impl App {
    /// Bouton de colonne triable : libellé et pictogramme de tri.
    fn column_button(self: &Rc<Self>, key: SortKey, text: &str) -> Button {
        let button = Button::new();
        button.add_css_class("th");
        set_class(&button, "sorted", self.st.borrow().sort == key);
        let row = hbox(6);
        let title = label(&text.to_uppercase(), &[]);
        title.set_hexpand(true);
        row.append(&title);
        row.append(&icon_with_stroke("sort", 12, 1.6));
        button.set_child(Some(&row));
        on_click(&button, self, move |app| app.sort_by(key));
        button
    }

    /// Reconstruit l'en-tête : case vide, quatre colonnes triables, case vide.
    pub fn render_table(self: &Rc<Self>) {
        let head = &self.ui.files.thead;
        clear(head);
        let (tag, size, date) = column_widths(self.set.borrow().full_dates);
        head.append(&fixed(34, &hbox(0)));
        let name = self.column_button(SortKey::Name, "Nom");
        name.set_hexpand(true);
        head.append(&name);
        let columns = if self.compact.get() {
            vec![(SortKey::Size, "Taille", 72)]
        } else {
            vec![(SortKey::Kind, "Type", tag), (SortKey::Size, "Taille", size), (SortKey::Modified, "Modifié", date)]
        };
        for (key, text, width) in columns {
            let column = self.column_button(key, text);
            column.set_halign(Align::Fill);
            head.append(&fixed(width, &column));
        }
        head.append(&fixed(36, &hbox(0)));
    }

    /// Trie par `key` : même colonne inverse le sens, autre colonne repart croissante (nom, type) ou décroissante.
    pub fn sort_by(self: &Rc<Self>, key: SortKey) {
        let ascending = if self.st.borrow().sort == key { !self.st.borrow().ascending } else { matches!(key, SortKey::Name | SortKey::Kind) };
        {
            let mut set = self.set.borrow_mut();
            (set.sort, set.ascending) = (key.letter().to_string(), ascending);
            set.save(&self.home);
        }
        {
            let mut st = self.st.borrow_mut();
            (st.sort, st.ascending, st.sel) = (key, ascending, -1);
        }
        self.render(None);
        self.show_preview();
    }
}
