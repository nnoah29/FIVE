//! Ouverture et fermeture de la page Paramètres.

use std::rc::Rc;

use crate::app::shared::App;

impl App {
    /// Ouvre la page Paramètres.
    pub fn open_settings(self: &Rc<Self>) {
        self.st.borrow_mut().settings_open = true;
        self.ui.stack.set_visible_child_name("settings");
        self.sync_toggles();
        self.render_settings();
    }

    /// Ferme la page Paramètres et revient aux fichiers.
    pub fn close_settings(self: &Rc<Self>) {
        self.st.borrow_mut().settings_open = false;
        self.ui.stack.set_visible_child_name("content");
        self.sync_toggles();
    }

    /// Ouvre ou ferme la page Paramètres.
    pub fn toggle_settings(self: &Rc<Self>) {
        if self.st.borrow().settings_open { self.close_settings() } else { self.open_settings() }
    }
}
