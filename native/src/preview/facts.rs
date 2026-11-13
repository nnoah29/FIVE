//! Fiche de renseignements : deux colonnes (libellé, valeur) dans un cadre.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{Grid, Label};

use crate::ui::widgets::label;

/// Grille de faits dont on peut ajouter des lignes plus tard (dimensions, durée).
pub struct Facts {
    pub grid: Grid,
    rows: Cell<i32>,
    keys: RefCell<Vec<String>>,
}

impl Facts {
    /// Fiche vide.
    pub fn new() -> Rc<Facts> {
        let grid = Grid::new();
        grid.add_css_class("sheet");
        Rc::new(Facts { grid, rows: Cell::new(0), keys: RefCell::new(Vec::new()) })
    }

    /// Ajoute une ligne ; un libellé déjà présent est ignoré.
    pub fn add(&self, key: &str, value: &str) {
        if self.keys.borrow().iter().any(|k| k == key) {
            return;
        }
        self.keys.borrow_mut().push(key.to_string());
        let row = self.rows.get();
        let name: Label = label(&key.to_uppercase(), &["fact-k"]);
        let text: Label = label(value, &["fact-v"]);
        text.set_hexpand(true);
        text.set_wrap(true);
        text.set_selectable(true);
        self.grid.attach(&name, 0, row, 1, 1);
        self.grid.attach(&text, 1, row, 1, 1);
        self.rows.set(row + 1);
    }
}
