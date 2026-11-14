//! Éditeur d'une règle (à compléter) : pour l'instant, annonce seulement qu'il arrive.

use std::rc::Rc;

use crate::app::shared::App;

impl App {
    /// Ouvre l'éditeur d'une règle existante (`Some(id)`) ou d'une nouvelle règle (`None`).
    pub fn open_rule_editor(self: &Rc<Self>, _id: Option<String>) {
        self.toast("L'éditeur de règles arrive");
    }
}
