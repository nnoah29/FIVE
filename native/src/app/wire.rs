//! Câblage des boutons fixes de la fenêtre.

use std::rc::Rc;
use std::time::Duration;

use gtk4::prelude::*;
use gtk4::{glib, EventControllerFocus};

use super::shared::{on_click, App};
use crate::ui::classes::set_class;

impl App {
    /// Câble la recherche : délai de 250 ms après la dernière frappe, puis relecture du dossier.
    fn wire_search(self: &Rc<Self>) {
        let weak = Rc::downgrade(self);
        self.ui.top.search.connect_changed(move |entry| {
            let Some(app) = weak.upgrade() else { return };
            let generation = app.search_gen.get() + 1;
            app.search_gen.set(generation);
            let (text, weak) = (entry.text().to_string(), Rc::downgrade(&app));
            glib::timeout_add_local_once(Duration::from_millis(250), move || {
                let Some(app) = weak.upgrade().filter(|a| a.search_gen.get() == generation) else { return };
                app.st.borrow_mut().query = text;
                let path = app.st.borrow().path.clone();
                app.load(path, None);
            });
        });
        let focus = EventControllerFocus::new();
        let (enter, leave) = (self.ui.top.root.clone(), self.ui.top.root.clone());
        focus.connect_enter(move |_| set_class(&enter, "focused", true));
        focus.connect_leave(move |_| set_class(&leave, "focused", false));
        self.ui.top.search.add_controller(focus);
    }

    /// Câble tous les boutons fixes : navigation, création, affichage, thème, paramètres, collage.
    pub fn wire(self: &Rc<Self>) {
        on_click(&self.ui.side.up, self, |app| app.go_up());
        on_click(&self.ui.side.new_folder, self, |app| app.create_folder());
        on_click(&self.ui.top.hidden, self, |app| app.toggle_hidden());
        on_click(&self.ui.top.theme, self, |app| app.toggle_theme());
        on_click(&self.ui.top.gear, self, |app| app.toggle_settings());
        on_click(&self.ui.files.view_list, self, |app| app.set_view("list"));
        on_click(&self.ui.files.view_grid, self, |app| app.set_view("grid"));
        on_click(&self.ui.files.paste, self, |app| app.paste_clip());
        on_click(&self.ui.work.close, self, |app| app.ui.work.root.set_visible(false));
        on_click(&self.ui.top.menu, self, |app| app.ui.side_scroll.set_visible(true));
        self.wire_search();
    }
}
