//! Page Paramètres : onglets et panneaux, à la place du contenu.

pub mod accent;
pub mod controls;
pub mod display;
pub mod folder_fields;
pub mod folders;
pub mod panels;
pub mod rules_actions;
pub mod rules_editor;
pub mod rules_history;
pub mod rules_panel;
pub mod rules_preview;
pub mod rules_row;
pub mod visibility;

use std::rc::Rc;

use gtk4::Button;
use gtk4::prelude::*;

use crate::app::shared::{App, on_click};
use crate::ui::buttons::icon_button;
use crate::ui::classes::{clear, set_class};
use crate::ui::widgets::{hbox, label, vbox};

const TABS: [(&str, &str); 6] =
    [("display", "Affichage"), ("files", "Fichiers"), ("folders", "Dossiers"), ("rules", "Règles"), ("keys", "Raccourcis"), ("system", "Système")];

impl App {
    /// Ligne du haut : titre et bouton de fermeture.
    fn settings_top(self: &Rc<Self>) -> gtk4::Box {
        let top = hbox(16);
        let title = label("PARAMÈTRES", &["h1"]);
        title.set_hexpand(true);
        let close = icon_button("x", &[]);
        on_click(&close, self, |app| app.close_settings());
        top.append(&title);
        top.append(&close);
        top
    }

    /// Onglets : Affichage, Fichiers, Dossiers, Règles, Raccourcis, Système.
    fn settings_tabs(self: &Rc<Self>) -> gtk4::Box {
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
        tabs
    }

    /// Cadre du panneau de l'onglet courant.
    fn settings_card(self: &Rc<Self>) -> gtk4::Box {
        let card = vbox(0);
        card.add_css_class("group-card");
        let panel = match self.st.borrow().tab.as_str() {
            "files" => self.files_panel(),
            "folders" => self.folders_panel(),
            "rules" => self.rules_panel(),
            "keys" => self.keys_panel(),
            "system" => self.system_panel(),
            _ => self.display_panel(),
        };
        let count = panel.len();
        for (at, widget) in panel.iter().enumerate() {
            set_class(widget, "last", at + 1 == count);
            card.append(widget);
        }
        card
    }

    /// Reconstruit la page : titre, onglets, panneau choisi.
    pub fn render_settings(self: &Rc<Self>) {
        clear(&self.ui.settings);
        self.ui.settings.append(&self.settings_top());
        self.ui.settings.append(&self.settings_tabs());
        self.ui.settings.append(&self.settings_card());
    }
}
