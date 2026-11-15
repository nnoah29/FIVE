//! Panneau d'aperçu du sélecteur : ce que contient l'élément sélectionné.

use gtk4::prelude::*;

use super::listing::Row;
use super::preview_parts::{entry_of, facts_box};
use crate::model::kinds::kind_of;
use crate::preview::media::fill_zone;
use crate::ui::widgets::vbox;

/// Hauteur fixe de la zone d'aperçu, quel que soit le contenu.
const ZONE_HEIGHT: i32 = 260;

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
    root.set_hexpand(false);
    root.set_overflow(gtk4::Overflow::Hidden);
    let body = vbox(14);
    body.add_css_class("chooser-preview-body");
    root.append(&body);
    Preview { root, body }
}

impl Preview {
    /// Montre l'aperçu de la ligne donnée, ou vide le panneau s'il n'y en a pas.
    pub fn show(&self, row: Option<&Row>) {
        while let Some(child) = self.body.first_child() {
            self.body.remove(&child);
        }
        let Some(row) = row else { return };
        let (zone, entry) = (vbox(8), entry_of(row));
        zone.add_css_class("preview");
        zone.add_css_class("none");
        zone.set_size_request(-1, ZONE_HEIGHT);
        zone.set_vexpand(false);
        zone.set_valign(gtk4::Align::Start);
        zone.set_overflow(gtk4::Overflow::Hidden);
        self.body.append(&zone);
        fill_zone(&zone, row.path.clone(), &entry, kind_of(&entry), None, ZONE_HEIGHT - 12, false);
        self.body.append(&facts_box(row));
    }
}
