//! Câblage du sélecteur : boutons, liste, champs et raccourcis clavier.

use std::rc::Rc;

use gtk4::gdk::{Key, ModifierType};
use gtk4::prelude::*;
use gtk4::{EventControllerKey, glib};

use super::request::Mode;
use super::window::Chooser;

impl Chooser {
    /// Relie un bouton à une action du sélecteur.
    fn on_button(self: &Rc<Self>, button: &gtk4::Button, action: impl Fn(&Rc<Chooser>) + 'static) {
        let weak = Rc::downgrade(self);
        button.connect_clicked(move |_| {
            if let Some(chooser) = weak.upgrade() {
                action(&chooser);
            }
        });
    }

    /// Câble les boutons, le chemin, le nom, le filtre et la fermeture.
    pub fn wire(self: &Rc<Self>) {
        self.on_button(&self.header.up, |c| c.go_up());
        self.on_button(&self.header.hidden, |c| c.toggle_hidden());
        self.on_button(&self.footer.cancel, |c| c.cancel());
        self.on_button(&self.footer.accept, |c| c.accept());
        self.wire_fields();
        self.wire_list();
        self.wire_keys();
    }

    /// Câble le champ du chemin, le nom du fichier, le filtre et la fermeture de la fenêtre.
    fn wire_fields(self: &Rc<Self>) {
        let weak = Rc::downgrade(self);
        self.header.path.connect_activate(move |entry| {
            if let Some(c) = weak.upgrade() {
                c.submit_path(&entry.text());
            }
        });
        let weak = Rc::downgrade(self);
        self.footer.name.connect_changed(move |_| weak.upgrade().iter().for_each(|c| c.refresh_accept()));
        let weak = Rc::downgrade(self);
        self.footer.name.connect_activate(move |_| weak.upgrade().iter().for_each(|c| c.accept()));
        let weak = Rc::downgrade(self);
        if let Some(menu) = &self.footer.filter {
            menu.connect_selected_notify(move |_| weak.upgrade().iter().for_each(|c| c.reload()));
        }
        let weak = Rc::downgrade(self);
        self.window.connect_close_request(move |_| {
            weak.upgrade().iter().for_each(|c| c.finish(None));
            glib::Propagation::Proceed
        });
    }

    /// Câble la liste : double-clic ou Entrée, et changement de sélection.
    fn wire_list(self: &Rc<Self>) {
        let weak = Rc::downgrade(self);
        self.list.connect_row_activated(move |_, row| weak.upgrade().iter().for_each(|c| c.activate_row(row.index() as usize)));
        let weak = Rc::downgrade(self);
        self.list.connect_selected_rows_changed(move |_| weak.upgrade().iter().for_each(|c| c.on_selection()));
    }

    /// Raccourcis : Échap annule, Ctrl L va au chemin, Alt ↑ monte, Ctrl H affiche les cachés.
    fn wire_keys(self: &Rc<Self>) {
        let (keys, weak) = (EventControllerKey::new(), Rc::downgrade(self));
        keys.connect_key_pressed(move |_, key, _, state| {
            let Some(c) = weak.upgrade() else { return glib::Propagation::Proceed };
            let (ctrl, alt) = (state.contains(ModifierType::CONTROL_MASK), state.contains(ModifierType::ALT_MASK));
            match key {
                Key::Escape => c.cancel(),
                Key::Return | Key::KP_Enter if c.list.selected_rows().is_empty() && c.request.mode == Mode::Folder => c.accept(),
                Key::l if ctrl => {
                    let _ = c.header.path.grab_focus();
                }
                Key::h if ctrl => c.toggle_hidden(),
                Key::Up if alt => c.go_up(),
                _ => return glib::Propagation::Proceed,
            }
            glib::Propagation::Stop
        });
        self.window.add_controller(keys);
    }
}
