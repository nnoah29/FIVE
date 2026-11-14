//! État vide : icône, message, action de sortie.

use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Justification};

use crate::app::shared::{App, on_click};
use crate::ui::icon::icon_with_stroke;
use crate::ui::widgets::{label, mono, text_button, vbox};

impl App {
    /// Message expliquant pourquoi la liste est vide.
    fn empty_message(&self, error: Option<&str>) -> String {
        let st = self.st.borrow();
        if let Some(text) = error {
            return text.to_string();
        }
        match st.virtual_view.as_deref() {
            Some("recent") => return "Aucun fichier récent dans cette racine.".into(),
            Some("favorites") => return "Aucun favori : ajoutez un dossier aux signets de votre gestionnaire de fichiers.".into(),
            Some(_) => return "Aucun partage réseau monté dans cette racine.".into(),
            None => {}
        }
        if !st.query.trim().is_empty() {
            return format!("Aucun nom ne contient « {} ».", st.query.trim());
        }
        if st.items.iter().any(|i| !i.is_dir) {
            "Aucun fichier de ce type dans ce dossier.".into()
        } else if st.items.is_empty() {
            "Ce dossier est vide.".into()
        } else {
            "Ce dossier ne contient que des sous-dossiers.".into()
        }
    }

    /// Action de sortie : effacer les filtres, ou remonter au dossier parent.
    fn empty_action(self: &Rc<Self>) -> Option<gtk4::Button> {
        let st = self.st.borrow();
        if st.virtual_view.is_some() {
            return None;
        }
        if !st.query.trim().is_empty() || st.filter != "all" {
            let button = text_button("Effacer les filtres", None, &[]);
            on_click(&button, self, |app| app.clear_filters());
            return Some(button);
        }
        (!st.path.is_empty()).then(|| {
            let button = text_button("Dossier parent", None, &[]);
            on_click(&button, self, |app| app.go_up());
            button
        })
    }

    /// Bloc centré de l'état vide.
    pub fn empty_state(self: &Rc<Self>, error: Option<&str>) -> GtkBox {
        let column = vbox(10);
        column.add_css_class("empty");
        column.set_valign(Align::Center);
        column.set_halign(Align::Center);
        column.set_vexpand(true);
        column.set_size_request(-1, 220);
        let glyph = icon_with_stroke("folder", 44, 1.6);
        glyph.set_halign(Align::Center);
        column.append(&glyph);
        let title = mono(if error.is_some() { "Lecture impossible" } else { "Aucun fichier" }, &[]);
        title.set_halign(Align::Center);
        title.set_xalign(0.5);
        column.append(&title);
        let text = label(&self.empty_message(error), &["empty-text"]);
        text.set_wrap(true);
        text.set_max_width_chars(34);
        text.set_justify(Justification::Center);
        text.set_xalign(0.5);
        text.set_halign(Align::Center);
        column.append(&text);
        if let Some(button) = self.empty_action() {
            button.set_halign(Align::Center);
            button.set_margin_top(8);
            column.append(&button);
        }
        column
    }
}
