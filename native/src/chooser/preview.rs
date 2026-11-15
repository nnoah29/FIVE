//! Panneau d'aperçu du sélecteur : la même fiche que le panneau de droite de la fenêtre principale.

use gtk4::prelude::*;

use super::listing::Row;
use super::preview_parts::entry_of;
use crate::model::settings::Settings;
use crate::preview::item::item_sheet;
use crate::ui::widgets::vbox;

/// Le panneau, son contenu et l'unité de taille choisie dans les réglages.
pub struct Preview {
    pub root: gtk4::ScrolledWindow,
    body: gtk4::Box,
    binary_units: bool,
}

/// Construit le panneau vide : mêmes classes et même largeur que le panneau de la fenêtre principale.
pub fn build_preview() -> Preview {
    let body = vbox(18);
    body.add_css_class("work-body");
    let root = gtk4::ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .min_content_width(360)
        .max_content_width(360)
        .propagate_natural_width(false)
        .child(&body)
        .build();
    root.add_css_class("work");
    root.set_size_request(360, -1);
    root.set_hexpand(false);
    let home = std::env::var_os("HOME").map(std::path::PathBuf::from).unwrap_or_default();
    Preview { root, body, binary_units: Settings::load(&home).binary_units }
}

impl Preview {
    /// Montre la fiche de la ligne donnée, ou vide le panneau s'il n'y en a pas.
    pub fn show(&self, row: Option<&Row>) {
        while let Some(child) = self.body.first_child() {
            self.body.remove(&child);
        }
        let Some(row) = row else { return };
        let (title, zone, facts) = item_sheet(&entry_of(row), row.path.clone(), self.binary_units);
        self.body.append(&title);
        self.body.append(&zone);
        self.body.append(&facts.grid);
    }
}
