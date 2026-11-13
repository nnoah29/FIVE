//! Message temporaire en bas de la fenêtre.

use std::rc::Rc;
use std::time::Duration;

use gtk4::glib;
use gtk4::prelude::*;

use super::shared::App;

impl App {
    /// Affiche un message pendant 2,2 secondes.
    pub fn toast(self: &Rc<Self>, text: &str) {
        self.ui.toast.set_text(text);
        self.ui.toast.add_css_class("on");
        let generation = self.toast_gen.get() + 1;
        self.toast_gen.set(generation);
        let (toast, app) = (self.ui.toast.clone(), Rc::downgrade(self));
        glib::timeout_add_local_once(Duration::from_millis(2200), move || {
            if app.upgrade().is_some_and(|a| a.toast_gen.get() == generation) {
                toast.remove_css_class("on");
            }
        });
    }
}
