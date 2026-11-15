//! Panneau d'aperçu du sélecteur : ce que contient l'élément sélectionné.

use gtk4::prelude::*;

use super::listing::Row;
use super::preview_parts::{facts_box, image_widget, text_excerpt, text_widget};
use crate::ui::icon::icon_with_stroke;
use crate::ui::widgets::vbox;

/// Le panneau et sa zone de contenu.
pub struct Preview {
    pub root: gtk4::Box,
    body: gtk4::Box,
}

/// Construit le panneau, vide, de largeur fixe.
pub fn build_preview() -> Preview {
    let root = vbox(0);
    root.add_css_class("chooser-preview");
    root.set_size_request(300, -1);
    let body = vbox(14);
    body.add_css_class("chooser-preview-body");
    root.append(&body);
    Preview { root, body }
}

/// Contenu visuel : dossier ou icône, image, ou début du texte.
fn visual(row: &Row) -> gtk4::Widget {
    let big = |name: &str| icon_with_stroke(name, 64, 1.3).upcast::<gtk4::Widget>();
    if row.is_dir {
        return big("folder");
    }
    image_widget(row).or_else(|| text_excerpt(&row.path).map(|t| text_widget(&t))).unwrap_or_else(|| big("file"))
}

impl Preview {
    /// Montre l'aperçu de la ligne donnée, ou vide le panneau s'il n'y en a pas.
    pub fn show(&self, row: Option<&Row>) {
        while let Some(child) = self.body.first_child() {
            self.body.remove(&child);
        }
        let Some(row) = row else { return };
        let picture = visual(row);
        picture.set_halign(gtk4::Align::Fill);
        self.body.append(&picture);
        self.body.append(&facts_box(row));
    }
}
