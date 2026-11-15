//! Fin du sélecteur : renvoyer le choix, annuler, et activer le bouton de validation.

use gtk4::prelude::*;
use std::path::PathBuf;
use std::rc::Rc;

use super::outcome::Choice;
use super::request::Mode;
use super::window::Chooser;

impl Chooser {
    /// Termine avec ces chemins et le filtre actif.
    pub fn finish_with(self: &Rc<Self>, paths: Vec<PathBuf>) {
        let filter = self.footer.filter.as_ref().map(|menu| menu.selected() as usize);
        self.finish(Some(Choice { paths, filter }));
    }

    /// Appelle une seule fois la fonction de fin, puis ferme la fenêtre.
    pub fn finish(self: &Rc<Self>, choice: Option<Choice>) {
        let done = self.done.borrow_mut().take();
        if let Some(done) = done {
            done(choice);
            self.window.close();
        }
    }

    /// Annule la sélection.
    pub fn cancel(self: &Rc<Self>) {
        self.finish(None);
    }

    /// Active le bouton selon le mode : une sélection pour « Ouvrir », un nom pour « Enregistrer ».
    pub fn refresh_accept(&self) {
        let ready = match self.request.mode {
            Mode::Open => !self.selected_rows().is_empty(),
            Mode::Folder => true,
            Mode::Save => !self.footer.name.text().trim().is_empty(),
        };
        self.footer.accept.set_sensitive(ready);
    }
}
