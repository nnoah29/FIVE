//! Sélection : simple, multiple (Ctrl, Maj), tout, ouverture et déplacement au clavier.

use std::rc::Rc;

use gtk4::Widget;
use gtk4::gdk::ModifierType;
use gtk4::prelude::*;

use five_engine::entry::Entry;

use super::shared::App;
use crate::ui::classes::set_class;

impl App {
    /// Copie des éléments sélectionnés (le verrou de l'état est relâché avant tout appel).
    pub fn picked(&self) -> Vec<Entry> {
        self.st.borrow().picked_items()
    }

    /// Widgets des lignes ou tuiles affichées, dans l'ordre de `shown`.
    pub fn row_widgets(&self) -> Vec<Widget> {
        let items = &self.ui.files.items;
        let mut out = Vec::new();
        let first = items.first_child();
        let mut cursor = if first.as_ref().is_some_and(|w| w.is::<gtk4::Grid>()) { first.and_then(|g| g.first_child()) } else { first };
        while let Some(widget) = cursor {
            cursor = widget.next_sibling();
            out.push(widget);
        }
        out
    }

    /// Met à jour la classe `selected` de chaque ligne ou tuile.
    pub fn paint_selection(&self) {
        let st = self.st.borrow();
        for (widget, item) in self.row_widgets().iter().zip(&st.shown) {
            set_class(widget, "selected", st.picked.contains(&item.path));
        }
    }

    /// Fait défiler la liste pour que la ligne `index` soit visible.
    fn scroll_to_row(&self, index: usize) {
        let Some(row) = self.row_widgets().get(index).cloned() else { return };
        let Some(bounds) = row.compute_bounds(&self.ui.files.items) else { return };
        let adjustment = self.ui.files.scroll.vadjustment();
        let (top, bottom) = (bounds.y() as f64, (bounds.y() + bounds.height()) as f64);
        if top < adjustment.value() {
            adjustment.set_value(top);
        } else if bottom > adjustment.value() + adjustment.page_size() {
            adjustment.set_value(bottom - adjustment.page_size());
        }
    }

    /// Met à jour l'ensemble sélectionné : Ctrl bascule, Maj étend depuis l'ancre, sinon remplace.
    fn apply_pick(&self, index: isize, mods: ModifierType) {
        let mut st = self.st.borrow_mut();
        st.sel = index;
        let item = usize::try_from(index).ok().and_then(|i| st.shown.get(i)).map(|i| i.path.clone());
        let (ctrl, shift) = (mods.contains(ModifierType::CONTROL_MASK) || mods.contains(ModifierType::META_MASK), mods.contains(ModifierType::SHIFT_MASK));
        match item {
            None => st.picked.clear(),
            Some(path) if ctrl => {
                if !st.picked.remove(&path) {
                    st.picked.insert(path);
                }
            }
            Some(_) if shift && st.anchor >= 0 => {
                let (a, b) = (st.anchor.min(index) as usize, st.anchor.max(index) as usize);
                st.picked = st.shown[a..=b.min(st.shown.len().saturating_sub(1))].iter().map(|i| i.path.clone()).collect();
            }
            Some(path) => st.picked = std::iter::once(path).collect(),
        }
        if !shift {
            st.anchor = index;
        }
    }

    /// Sélectionne l'élément `index` ; Ctrl bascule, Maj étend depuis l'ancre, `-1` vide la sélection.
    pub fn select(self: &Rc<Self>, index: isize, mods: ModifierType) {
        self.apply_pick(index, mods);
        self.paint_selection();
        if index >= 0 {
            self.scroll_to_row(index as usize);
        }
        self.show_preview();
        if !self.st.borrow().picked.is_empty() {
            self.open_work();
        }
    }
}
