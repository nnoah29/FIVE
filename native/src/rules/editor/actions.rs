//! Boutons de l'éditeur : enregistrer, aperçu, annuler.

use std::rc::Rc;

use five_engine::rules::api_write::{create_rule, update_rule};
use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox};
use serde_json::json;

use super::draft::cleaned;
use super::state::Editor;
use crate::ui::widgets::{hbox, text_button};

impl Editor {
    /// Enregistre la règle ; en cas de refus, la raison s'affiche dans l'éditeur.
    fn save(self: &Rc<Self>) {
        let Some(app) = self.app() else { return };
        let body = cleaned(&self.draft.borrow());
        let result = match &self.id {
            Some(id) => update_rule(&app.root, &app.rules_dir(), id, &body),
            None => create_rule(&app.root, &app.rules_dir(), &body),
        };
        match result {
            Ok(_) => {
                app.close_modal(&self.modal);
                app.render_settings();
                app.toast("Règle enregistrée : elle est en essai, consultez son aperçu pour l'activer");
            }
            Err(refus) => self.notice.set_text(&refus.message),
        }
    }

    /// Pied de page : annuler, aperçu de la règle en cours d'édition, enregistrer.
    pub fn footer(self: &Rc<Self>) -> GtkBox {
        let row = hbox(8);
        row.set_halign(Align::End);
        let (cancel, preview, save) =
            (text_button("Annuler", None, &[]), text_button("Aperçu", Some("eye"), &[]), text_button("Enregistrer", Some("check"), &["go"]));
        let (weak, modal) = (Rc::downgrade(self), self.modal.clone());
        cancel.connect_clicked(move |_| {
            if let Some(app) = weak.upgrade().and_then(|e| e.app()) {
                app.close_modal(&modal);
            }
        });
        let weak = Rc::downgrade(self);
        preview.connect_clicked(move |_| {
            if let Some((ed, app)) = weak.upgrade().and_then(|ed| ed.app().map(|app| (ed, app))) {
                app.open_preview_of(json!({"rule": cleaned(&ed.draft.borrow())}), None);
            }
        });
        let weak = Rc::downgrade(self);
        save.connect_clicked(move |_| {
            if let Some(ed) = weak.upgrade() {
                ed.save();
            }
        });
        for button in [cancel, preview, save] {
            row.append(&button);
        }
        row
    }
}
