//! Emplacement vide affiché à la place des cartes quand le dossier n'a pas de sous-dossier.

use gtk4::Align;
use gtk4::prelude::*;

use crate::app::shared::App;
use crate::ui::widgets::{label, vbox};

impl App {
    /// Emplacement vide de la même taille qu'une carte, pour que la page ne remonte pas sans sous-dossier.
    pub(super) fn empty_card(&self) -> gtk4::Box {
        let card = vbox(0);
        card.add_css_class("card-empty");
        card.set_size_request(self.card_width(), -1);
        let text = label("Aucun sous-dossier", &["dim"]);
        text.set_halign(Align::Center);
        text.set_valign(Align::Center);
        text.set_vexpand(true);
        card.append(&text);
        card
    }
}
