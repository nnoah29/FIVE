//! Panneau de droite : espace du dossier, aperçu d'un élément, actions sur une sélection.

pub mod batch;
pub mod dups;
pub mod dups_rows;
pub mod facts;
pub mod fill;
pub mod gauge;
pub mod images;
pub mod item;
pub mod media;
pub mod space;
pub mod usage;

use std::rc::Rc;

use gtk4::prelude::*;

use crate::app::shared::App;
use crate::ui::classes::clear;

impl App {
    /// Reconstruit le panneau de droite selon la sélection : espace, un élément ou plusieurs.
    pub fn show_preview(self: &Rc<Self>) {
        clear(&self.ui.work.body);
        let chosen = self.st.borrow().picked_items();
        let widgets = match chosen.len() {
            0 => self.space_panel(),
            1 => self.item_panel(&chosen[0]),
            _ => self.batch_panel(&chosen),
        };
        widgets.iter().for_each(|w| self.ui.work.body.append(w));
    }
}
