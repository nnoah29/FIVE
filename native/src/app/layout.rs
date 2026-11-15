//! Disposition adaptative : panneau de droite et barre latérale en tiroirs sur petite fenêtre.

use std::rc::Rc;

use gtk4::prelude::*;

use super::shared::App;
use crate::ui::classes::{set_button_label_visible, set_class};

/// Largeur en dessous de laquelle le panneau de droite devient un tiroir.
pub const MEDIUM: i32 = 1100;

/// Largeur en dessous de laquelle la barre latérale devient un tiroir.
pub const NARROW: i32 = 760;

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
        if self.narrow_side() {
            self.ui.side_scroll.set_visible(false);
            self.ui.scrim.set_visible(false);
        }
    }

    /// Adapte la disposition à la largeur de la fenêtre.
    pub fn apply_layout(self: &Rc<Self>) {
        let narrow = self.narrow_work();
        let narrow_side = self.narrow_side();
        self.place_side(narrow_side);
        if self.compact.replace(narrow_side) != narrow_side {
            self.render(None);
        } else {
            self.render_cards();
            if self.set.borrow().view == "grid" {
                self.render_items(None);
            }
        }
        self.ui.top.menu.set_visible(narrow_side);
        self.ui.work.close.set_visible(narrow);
        self.reserve_panel_space(narrow);
        let has_pick = !self.st.borrow().picked.is_empty();
        self.ui.work.root.set_visible(!narrow || has_pick);
        self.ui.top.user.set_visible(!narrow_side);
        self.ui.top.who.set_visible(!narrow);
        let small = self.ui.window.width() > 0 && self.ui.window.width() <= 900;
        set_button_label_visible(&self.ui.top.gear, !small);
        set_class(&self.ui.top.root, "small", small);
    }
}
