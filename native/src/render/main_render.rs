//! Rendu complet : filtre et trie les éléments, puis reconstruit chaque région.

use std::rc::Rc;

use gtk4::prelude::*;

use crate::app::shared::App;
use crate::model::kinds::kind_of;
use crate::model::sorting::sort_items;
use crate::ui::classes::clear;

impl App {
    /// Recalcule la liste affichée (filtre, tri) et redessine toute la fenêtre.
    pub fn render(self: &Rc<Self>, error: Option<&str>) {
        {
            let mut st = self.st.borrow_mut();
            if st.sel < 0 {
                st.picked.clear();
            }
            let all = st.list_all();
            let kept: Vec<_> = st.items.iter().filter(|i| (all || !i.is_dir) && (st.filter == "all" || kind_of(i) == st.filter)).cloned().collect();
            st.shown = sort_items(&kept, st.sort, st.ascending);
        }
        self.render_side();
        self.render_head();
        self.render_cards();
        self.render_table();
        self.render_items(error);
        self.update_paste();
    }

    /// Reconstruit la liste (lignes ou tuiles), ou l'état vide s'il n'y a rien à montrer.
    pub fn render_items(self: &Rc<Self>, error: Option<&str>) {
        clear(&self.ui.files.items);
        let (empty, grid) = (self.st.borrow().shown.is_empty(), self.set.borrow().view == "grid");
        if empty {
            self.ui.files.items.append(&self.empty_state(error));
        } else if grid {
            self.ui.files.items.append(&self.build_tiles());
        } else {
            self.build_rows().iter().for_each(|row| self.ui.files.items.append(row));
        }
    }
}
