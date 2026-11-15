//! Placement d'un menu contextuel : sous l'ancre, sous le pointeur ou aligné à droite.

use std::rc::Rc;

use gtk4::gdk::Rectangle;
use gtk4::prelude::*;
use gtk4::{Align, Popover, Widget};

use super::popover::MenuEntry;
use super::shared::App;

/// Où le menu s'ouvre par rapport à son ancre.
#[derive(Clone, Copy)]
pub enum MenuPlace {
    Below,
    Pointer(f64, f64),
    BelowEnd,
}

impl MenuPlace {
    /// Règle l'alignement et le point visé du menu.
    pub fn apply(self, popover: &Popover) {
        match self {
            MenuPlace::Below => {}
            MenuPlace::Pointer(x, y) => {
                popover.set_halign(Align::Start);
                popover.set_pointing_to(Some(&Rectangle::new(x as i32, y as i32 + 4, 1, 1)));
            }
            MenuPlace::BelowEnd => popover.set_halign(Align::End),
        }
    }
}

impl App {
    /// Ouvre un menu sous `anchor`, comme avant.
    pub fn popover(self: &Rc<Self>, anchor: &impl IsA<Widget>, entries: Vec<MenuEntry>) {
        self.popover_at(anchor, MenuPlace::Below, entries);
    }
}
