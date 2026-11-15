//! Réglage « Sélecteur de fichiers du système » : FIVE ouvre sa fenêtre simple à la place du dialogue par défaut.

use std::rc::Rc;

use gtk4::Widget;
use gtk4::prelude::*;

use super::controls::setting_row;
use crate::app::shared::App;
use crate::chooser::portal_enable::{chooser_enabled, set_enabled};
use crate::ui::controls::tick;

impl App {
    /// Ligne du réglage : une case qui active FIVE comme sélecteur de fichiers de vos applications.
    pub fn chooser_row(self: &Rc<Self>) -> Widget {
        let on = chooser_enabled(&self.home);
        let (button, weak) = (tick(on), Rc::downgrade(self));
        button.connect_clicked(move |_| weak.upgrade().iter().for_each(|app| app.switch_chooser(!on)));
        let text = "Les boîtes « Ouvrir » et « Enregistrer sous » de votre navigateur et de vos autres applications s'ouvrent dans FIVE. Prend effet à la prochaine ouverture d'une boîte.";
        setting_row("Sélecteur de fichiers du système", Some(text), &button, false).upcast()
    }

    /// Active ou rétablit le sélecteur de fichiers, puis le dit à l'utilisateur.
    fn switch_chooser(self: &Rc<Self>, enable: bool) {
        match set_enabled(&self.home, enable) {
            Ok(_) => self.toast(if enable { "FIVE est maintenant votre sélecteur de fichiers" } else { "Sélecteur de fichiers par défaut rétabli" }),
            Err(error) => self.toast(&format!("Impossible de modifier la configuration : {error}")),
        }
        self.render_settings();
    }
}
