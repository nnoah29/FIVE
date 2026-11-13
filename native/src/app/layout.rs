//! Disposition adaptative : panneau de droite et barre latérale en tiroirs sur petite fenêtre.

use std::rc::Rc;

use gtk4::prelude::*;

use super::shared::App;

/// Largeur en dessous de laquelle le panneau de droite devient un tiroir.
pub const MEDIUM: i32 = 1100;

impl App {
    /// Indique si la fenêtre est assez étroite pour que le panneau de droite soit un tiroir.
    pub fn narrow_work(&self) -> bool {
        self.ui.window.width() > 0 && self.ui.window.width() <= MEDIUM
    }

    /// Montre le panneau de droite (utile en mode tiroir).
    pub fn open_work(self: &Rc<Self>) {
        self.ui.work.root.set_visible(true);
    }

    /// Ferme le panneau de droite en mode tiroir et la barre latérale ouverte.
    pub fn close_drawers(self: &Rc<Self>) {
        if self.narrow_work() {
            self.ui.work.root.set_visible(false);
        }
    }

    /// Adapte la disposition à la largeur de la fenêtre.
    pub fn apply_layout(self: &Rc<Self>) {
        let narrow = self.narrow_work();
        self.ui.work.close.set_visible(narrow);
        self.ui.files.root.set_margin_end(if narrow { 0 } else { 360 });
        let has_pick = !self.st.borrow().picked.is_empty();
        self.ui.work.root.set_visible(!narrow || has_pick);
        self.ui.top.user.set_visible(self.ui.window.width() > 760);
    }
}
