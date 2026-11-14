//! Tuiles de la vue grille : vignette ou icône, nom, taille.

use std::rc::Rc;

use five_engine::entry::Entry;
use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Grid, Overlay, Picture};

use super::press::attach_press;
use crate::app::shared::App;
use crate::model::format::format_size;
use crate::model::kinds::{kind_icon, kind_of};
use crate::ui::icon::icon_with_stroke;
use crate::ui::widgets::{label, vbox};

impl App {
    /// Cadre de la vignette : image chargée en arrière-plan pour les images (en surimpression, pour ne pas peser sur la largeur), icône sinon.
    fn thumb_for(self: &Rc<Self>, item: &Entry) -> Overlay {
        let base = GtkBox::new(gtk4::Orientation::Horizontal, 0);
        base.set_size_request(-1, 92);
        let frame = Overlay::new();
        frame.add_css_class("thumb");
        frame.set_child(Some(&base));
        frame.set_overflow(gtk4::Overflow::Hidden);
        let (kind, thumbs) = (kind_of(item), self.set.borrow().thumbs);
        if kind == "image" && thumbs && item.size < 30_000_000 {
            let picture = Picture::new();
            picture.set_content_fit(gtk4::ContentFit::Cover);
            frame.add_overlay(&picture);
            self.load_thumb(&item.path, picture);
        } else {
            let glyph = icon_with_stroke(if item.is_link { "link" } else { kind_icon(kind) }, 30, 1.6);
            frame.add_overlay(&glyph);
        }
        frame
    }

    /// Tuile d'un élément.
    fn tile_for(self: &Rc<Self>, item: &Entry, index: usize) -> GtkBox {
        let tile = vbox(8);
        tile.add_css_class("tile");
        if self.st.borrow().picked.contains(&item.path) {
            tile.add_css_class("selected");
        }
        tile.append(&self.thumb_for(item));
        let name = label(&item.name, &["name"]);
        name.set_ellipsize(gtk4::pango::EllipsizeMode::End);
        name.set_max_width_chars(1);
        name.set_hexpand(true);
        name.set_halign(Align::Fill);
        name.set_tooltip_text(Some(&item.name));
        let size = if item.is_dir { "Dossier".to_string() } else { format_size(item.size, self.set.borrow().binary_units) };
        tile.append(&name);
        tile.append(&label(&size, &["meta"]));
        attach_press(self, &tile, item, index);
        tile
    }

    /// Nombre de colonnes de la grille : comme `repeat(auto-fill, minmax(132px, 1fr))`, avec 8 px d'écart.
    fn grid_columns(&self) -> i32 {
        let measured = self.ui.files.scroll.width();
        let available = if measured > 100 { measured } else { self.ui.window.width() - 248 - 360 - 56 };
        ((available + 8) / (132 + 8)).max(1)
    }

    /// Grille de toutes les tuiles, à colonnes de largeur égale.
    pub fn build_tiles(self: &Rc<Self>) -> Grid {
        let grid = Grid::new();
        grid.set_column_homogeneous(true);
        grid.set_column_spacing(8);
        grid.set_row_spacing(8);
        grid.set_valign(Align::Start);
        grid.set_margin_top(14);
        let (shown, columns) = (self.st.borrow().shown.clone(), self.grid_columns());
        for (index, item) in shown.iter().enumerate() {
            let tile = self.tile_for(item, index);
            grid.attach(&tile, index as i32 % columns, index as i32 / columns, 1, 1);
        }
        grid
    }
}
