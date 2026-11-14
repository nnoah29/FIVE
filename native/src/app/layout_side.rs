//! Barre latérale adaptative : dans la page sur grande fenêtre, en tiroir sur fenêtre étroite.

use std::rc::Rc;

use gtk4::prelude::*;

use super::layout::NARROW;
use super::shared::App;

impl App {
    /// Indique si la fenêtre est assez étroite pour que la barre latérale devienne un tiroir.
    pub fn narrow_side(&self) -> bool {
        self.ui.window.width() > 0 && self.ui.window.width() <= NARROW
    }

    /// Montre la barre latérale en tiroir, sur un voile sombre.
    pub fn open_side(self: &Rc<Self>) {
        if !self.narrow_side() {
            return;
        }
        self.ui.side_scroll.set_visible(true);
        self.ui.scrim.set_visible(true);
    }

    /// Place la barre latérale dans la page (large) ou en tiroir par-dessus (étroit).
    pub fn place_side(self: &Rc<Self>, narrow: bool) {
        let in_page = self.ui.side_scroll.parent().is_some_and(|p| p == self.ui.page.clone().upcast::<gtk4::Widget>());
        if narrow && in_page {
            self.ui.page.remove(&self.ui.side_scroll);
            self.ui.side_scroll.set_halign(gtk4::Align::Start);
            self.ui.side_scroll.set_size_request(300, -1);
            self.ui.overlay.add_overlay(&self.ui.side_scroll);
            self.ui.side_scroll.set_visible(false);
        } else if !narrow && !in_page {
            self.ui.overlay.remove_overlay(&self.ui.side_scroll);
            self.ui.side_scroll.set_halign(gtk4::Align::Fill);
            self.ui.side_scroll.set_size_request(248, -1);
            self.ui.page.prepend(&self.ui.side_scroll);
            self.ui.side_scroll.set_visible(true);
            self.ui.scrim.set_visible(false);
        }
    }
}
