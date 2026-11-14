//! Aperçu d'un élément : choix de la méthode selon la famille et l'extension.

use std::rc::Rc;

use five_engine::entry::Entry;
use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, PolicyType, ScrolledWindow, TextView};

use super::facts::Facts;
use crate::app::shared::App;
use crate::model::kinds::{extension, kind_icon};
use crate::ui::classes::clear;
use crate::ui::icon::icon_with_stroke;
use crate::ui::widgets::mono;

/// Indique si le nom est celui d'une archive que le moteur sait lister.
pub fn is_archive(name: &str) -> bool {
    let lower = name.to_lowercase();
    ["zip", "jar", "tar", "tgz", "tar.gz"].iter().any(|ext| lower.ends_with(&format!(".{ext}")))
}

/// Remplace le contenu de la zone par une icône et un message.
pub fn none_state(zone: &GtkBox, kind: &str, text: &str) {
    clear(zone);
    zone.add_css_class("none");
    zone.set_valign(Align::Center);
    let glyph = icon_with_stroke(kind_icon(kind), 44, 1.6);
    glyph.set_halign(Align::Center);
    zone.append(&glyph);
    let note = mono(text, &[]);
    note.set_halign(Align::Center);
    note.set_wrap(true);
    note.set_max_width_chars(1);
    note.set_justify(gtk4::Justification::Center);
    zone.append(&note);
}

/// Montre un texte à chasse fixe dans une zone défilante de hauteur limitée.
pub fn show_text(zone: &GtkBox, text: &str, height: i32) {
    clear(zone);
    zone.remove_css_class("none");
    let body = TextView::new();
    body.add_css_class("preview-text");
    body.buffer().set_text(text);
    body.set_editable(false);
    body.set_cursor_visible(false);
    body.set_focusable(false);
    body.set_wrap_mode(gtk4::WrapMode::Char);
    let scroll = ScrolledWindow::builder().hscrollbar_policy(PolicyType::Never).max_content_height(height).propagate_natural_height(true).child(&body).build();
    scroll.set_hexpand(true);
    zone.append(&scroll);
}

impl App {
    /// Remplit la zone d'aperçu de `item` ; `facts` reçoit les dimensions ou durées découvertes ; `height` borne la zone.
    pub fn fill_preview(self: &Rc<Self>, zone: &GtkBox, item: &Entry, kind: &str, facts: Option<Rc<Facts>>, height: i32) {
        let Ok(full) = self.root.resolve(&item.path) else { return };
        if item.is_dir {
            return none_state(zone, kind, "Dossier");
        }
        match kind {
            "image" => self.fill_image(zone, full, facts, height),
            "video" | "audio" => self.fill_media(zone, full, kind, facts, height),
            _ if is_archive(&item.name) => self.fill_archive(zone, full, height),
            _ if extension(&item.name) == "pdf" => self.fill_pdf(zone, full, height),
            _ if item.size > 50_000_000 => none_state(zone, kind, "Aperçu indisponible : fichier volumineux"),
            _ => self.fill_text(zone, full, kind, height),
        }
    }
}
