//! Réglages à bascule rapide : fichiers cachés, thème clair ou sombre, disposition.

use std::rc::Rc;

use super::shared::App;
use crate::ui::scheme::is_dark;

impl App {
    /// Affiche ou masque les fichiers cachés, avec un message.
    pub fn toggle_hidden(self: &Rc<Self>) {
        let now = !self.set.borrow().hidden;
        self.change_setting(|s| s.hidden = now);
        self.toast(if now { "Fichiers cachés affichés" } else { "Fichiers cachés masqués" });
    }

    /// Bascule entre thème clair et sombre.
    pub fn toggle_theme(self: &Rc<Self>) {
        let dark = is_dark(&self.set.borrow());
        self.change_setting(|s| {
            s.theme = if dark { "light" } else { "dark" }.to_string();
            s.wallpaper = false;
        });
    }

    /// Choisit l'affichage en liste ou en grille.
    pub fn set_view(self: &Rc<Self>, view: &str) {
        let view = view.to_string();
        self.change_setting(|s| s.view = view);
    }
}
