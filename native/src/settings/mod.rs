//! Page Paramètres : onglets et panneaux, à la place du contenu.

pub mod accent;
pub mod controls;
pub mod panels;

use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::Button;

use crate::app::shared::{on_click, App};
use crate::ui::classes::{clear, set_class};
use crate::ui::widgets::{hbox, icon_button, label, vbox};

const TABS: [(&str, &str); 4] = [("display", "Affichage"), ("files", "Fichiers"), ("keys", "Raccourcis"), ("system", "Système")];

impl App {
    /// Reconstruit la page : titre, bouton de fermeture, onglets, panneau choisi.
    pub fn render_settings(self: &Rc<Self>) {
        let page = &self.ui.settings;
        clear(page);
        let top = hbox(16);
        let title = label("PARAMÈTRES", &["h1"]);
        title.set_hexpand(true);
        let close = icon_button("x", &[]);
        on_click(&close, self, |app| app.close_settings());
        top.append(&title);
        top.append(&close);
        let tabs = hbox(28);
        tabs.add_css_class("tabs");
        let current = self.st.borrow().tab.clone();
        for (id, text) in TABS {
            let tab = Button::new();
            tab.set_child(Some(&label(&text.to_uppercase(), &[])));
            tab.add_css_class("tab");
            set_class(&tab, "active", current == id);
            on_click(&tab, self, move |app| {
                app.st.borrow_mut().tab = id.to_string();
                app.render_settings();
            });
            tabs.append(&tab);
        }
        let card = vbox(0);
        card.add_css_class("group-card");
        let panel = match current.as_str() {
            "files" => self.files_panel(),
            "keys" => self.keys_panel(),
            "system" => self.system_panel(),
            _ => self.display_panel(),
        };
        let count = panel.len();
        for (at, widget) in panel.iter().enumerate() {
            if at + 1 == count {
                widget.add_css_class("last");
            }
            card.append(widget);
        }
        page.append(&top);
        page.append(&tabs);
        page.append(&card);
    }

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
