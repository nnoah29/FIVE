//! Panneau Espace : jauge du dossier, outils d'analyse, répartition par type ou occupation.

use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{Align, Button, Widget};

use super::gauge::gauge;
use crate::app::shared::{App, on_click};
use crate::model::format::{format_size, group_digits};
use crate::model::kinds::{kind_icon, kind_of};
use crate::ui::classes::centered;
use crate::ui::icon::icon;
use crate::ui::widgets::{hbox, label, text_button, vbox};

/// Taille totale des fichiers du dossier par famille : images, vidéos, audio, documents, autres.
fn sizes_by_kind(app: &App) -> [u64; 5] {
    let mut sizes = [0u64; 5];
    for item in app.st.borrow().items.iter().filter(|i| !i.is_dir) {
        let slot = match kind_of(item) {
            "image" => 0,
            "video" => 1,
            "audio" => 2,
            "doc" => 3,
            _ => 4,
        };
        sizes[slot] += item.size;
    }
    sizes
}

/// Ligne de répartition : pictogramme, libellé, taille.
fn kind_row(icon_name: &str, text: &str, size: &str) -> gtk4::Box {
    let row = hbox(14);
    row.add_css_class("kind");
    let glyph = centered(34, 34, &icon(icon_name));
    glyph.add_css_class("glyph");
    let name = label(text, &["kind-name"]);
    name.set_hexpand(true);
    name.set_valign(Align::Center);
    row.append(&glyph);
    row.append(&name);
    row.append(&label(size, &["kind-size"]));
    row
}

impl App {
    /// Bloc « Dans ce dossier » : taille par famille de fichiers.
    fn kinds_block(&self) -> gtk4::Box {
        let units = self.set.borrow().binary_units;
        let sizes = sizes_by_kind(self);
        let block = vbox(0);
        block.append(&label("DANS CE DOSSIER", &["kinds-title"]));
        for (at, (name, text)) in [("image", "Images"), ("video", "Vidéos"), ("audio", "Audio"), ("doc", "Documents"), ("rest", "Autres")].iter().enumerate() {
            let glyph = if *name == "rest" { "file" } else { kind_icon(name) };
            block.append(&kind_row(glyph, text, &format_size(sizes[at], units)));
        }
        block
    }

    /// Les deux boutons d'outils, désactivés hors d'un dossier ordinaire.
    fn space_tools(self: &Rc<Self>) -> gtk4::Box {
        let tools = vbox(8);
        tools.set_margin_top(16);
        tools.set_margin_bottom(4);
        let off = self.st.borrow().virtual_view.is_some();
        let usage: Button = text_button("Analyser l'occupation", Some("archive"), &[]);
        let duplicates: Button = text_button("Chercher les doublons", Some("copy"), &[]);
        on_click(&usage, self, |app| app.analyse_usage());
        on_click(&duplicates, self, |app| app.find_duplicates());
        for button in [&usage, &duplicates] {
            button.set_sensitive(!off);
            button.set_halign(Align::Fill);
            tools.append(button);
        }
        tools
    }

    /// Panneau Espace complet, sous forme de widgets à placer dans le corps du panneau.
    pub fn space_panel(self: &Rc<Self>) -> Vec<Widget> {
        let (units, total, free) = (self.set.borrow().binary_units, self.st.borrow().total, self.st.borrow().free);
        let direct: u64 = sizes_by_kind(self).iter().sum();
        let usage = self.st.borrow().usage.clone().filter(|u| u.path == self.st.borrow().path).and_then(|u| u.data);
        let here = usage.as_ref().and_then(|d| d["total"].as_u64()).unwrap_or(direct);
        let (folders, files) = {
            let st = self.st.borrow();
            let folders = st.items.iter().filter(|i| i.is_dir).count();
            (folders, st.items.len() - folders)
        };
        let caption = if usage.is_some() {
            "avec les sous-dossiers".to_string()
        } else {
            format!(
                "{} fichier{} · {} dossier{}",
                group_digits(files as u64),
                if files > 1 { "s" } else { "" },
                group_digits(folders as u64),
                if folders > 1 { "s" } else { "" }
            )
        };
        let ratio = if total > 0 { here as f64 / total as f64 } else { 0.0 };
        let disk = label(&format!("Disque : {} libres sur {}", format_size(free, units), format_size(total, units)), &["disk-line"]);
        let lower: Widget = if self.st.borrow().usage.as_ref().is_some_and(|u| u.path == self.st.borrow().path) {
            self.usage_block().upcast()
        } else {
            self.kinds_block().upcast()
        };
        vec![label("ESPACE", &["h2"]).upcast(), gauge(ratio, &format_size(here, units), &caption).upcast(), disk.upcast(), self.space_tools().upcast(), lower]
    }
}
