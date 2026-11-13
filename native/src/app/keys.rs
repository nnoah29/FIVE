//! Raccourcis clavier de la fenêtre.

use std::rc::Rc;

use gtk4::gdk::{Key, ModifierType};
use gtk4::glib::Propagation;
use gtk4::prelude::*;
use gtk4::{EventControllerKey, PropagationPhase, Text, TextView};

use super::shared::App;

impl App {
    /// Indique si le focus est dans un champ de saisie.
    fn typing(&self) -> bool {
        GtkWindowExt::focus(&self.ui.window).is_some_and(|w| w.is::<Text>() || w.is::<TextView>())
    }

    /// Échap : ferme ce qui est au premier plan, dans l'ordre (modale, paramètres, menu, recherche, panneau, sélection).
    fn on_escape(self: &Rc<Self>) {
        if self.modal_open() {
            return self.close_top_modal();
        }
        if self.st.borrow().settings_open {
            return self.close_settings();
        }
        if self.menu.borrow().is_some() {
            return self.close_menu();
        }
        if !self.ui.top.search.text().is_empty() {
            self.ui.top.search.set_text("");
            self.st.borrow_mut().query.clear();
            return self.reload();
        }
        if self.narrow_work() && self.ui.work.root.is_visible() {
            self.ui.work.root.set_visible(false);
        } else if self.st.borrow().sel >= 0 {
            self.select(-1, ModifierType::empty());
        }
        GtkWindowExt::set_focus(&self.ui.window, None::<&gtk4::Widget>);
    }

    /// Raccourcis avec Ctrl : palette, tout sélectionner, copier, couper, coller.
    fn on_control(self: &Rc<Self>, key: Key) -> bool {
        let letter = key.to_unicode().map(|c| c.to_ascii_lowercase());
        match letter {
            Some('p' | 'k') => self.open_palette(),
            Some(c) if !self.typing() && !self.st.borrow().settings_open && "acxv".contains(c) => match c {
                'a' => self.select_all(),
                'c' => self.set_clip("copy", &self.st.borrow().picked_items()),
                'x' => self.set_clip("move", &self.st.borrow().picked_items()),
                _ => self.paste_clip(),
            },
            _ => return false,
        }
        true
    }

    /// Raccourcis simples hors saisie : navigation, ouverture, suppression, aperçu, renommage.
    fn on_plain(self: &Rc<Self>, key: Key) -> bool {
        if self.st.borrow().settings_open && key != Key::comma {
            return false;
        }
        match key {
            Key::slash => {
                self.ui.top.search.grab_focus();
            }
            Key::j | Key::Down => self.move_sel(1),
            Key::k | Key::Up => self.move_sel(-1),
            Key::Return | Key::KP_Enter => self.open_index(self.st.borrow().sel.max(0) as usize),
            Key::BackSpace | Key::h => self.go_up(),
            Key::period => self.toggle_hidden(),
            Key::c => self.create_folder(),
            Key::comma => self.toggle_settings(),
            Key::Delete | Key::numbersign => self.delete_items(self.st.borrow().picked_items()),
            Key::space => self.quick_look(None),
            Key::F2 => self.rename_items(self.st.borrow().picked_items()),
            _ => return false,
        }
        true
    }

    /// Installe le gestionnaire de touches sur la fenêtre (phase de capture).
    pub fn wire_keys(self: &Rc<Self>) {
        let keys = EventControllerKey::new();
        keys.set_propagation_phase(PropagationPhase::Capture);
        let weak = Rc::downgrade(self);
        keys.connect_key_pressed(move |_, key, _, state| {
            let Some(app) = weak.upgrade() else { return Propagation::Proceed };
            if key == Key::Escape {
                app.on_escape();
                return Propagation::Stop;
            }
            if app.modal_open() {
                return Propagation::Proceed;
            }
            let control = state.contains(ModifierType::CONTROL_MASK) && !state.contains(ModifierType::ALT_MASK);
            let handled = if control {
                app.on_control(key)
            } else if app.typing() || state.contains(ModifierType::ALT_MASK) {
                false
            } else {
                app.on_plain(key)
            };
            if handled { Propagation::Stop } else { Propagation::Proceed }
        });
        self.ui.window.add_controller(keys);
    }
}
