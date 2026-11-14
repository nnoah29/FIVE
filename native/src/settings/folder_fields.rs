//! Champs d'une ligne de l'onglet « Dossiers » : nom et chemin modifiables.

use std::path::Path;
use std::rc::Rc;

use gtk4::Entry;
use gtk4::prelude::*;

use crate::app::shared::App;
use crate::model::settings::Settings;
use crate::model::side_folders::{SideFolder, shown_label};
use crate::ui::classes::set_class;

/// Applique une modification au dossier épinglé de rang `at`, sans redessiner la page (la saisie en cours est conservée).
fn edit_at(app: &Rc<App>, at: usize, change: impl FnOnce(&mut SideFolder)) {
    app.change_setting_quiet(move |s: &mut Settings| {
        if let Some(folder) = s.side_folders.as_mut().and_then(|list| list.get_mut(at)) {
            change(folder);
        }
    });
}

impl App {
    /// Champ du nom affiché ; vide, la barre latérale montre le nom du dossier.
    pub fn name_entry(self: &Rc<Self>, at: usize, folder: &SideFolder) -> Entry {
        let entry = Entry::new();
        entry.add_css_class("hex");
        entry.set_text(&folder.label);
        entry.set_placeholder_text(Some(&shown_label(folder)));
        entry.set_width_chars(18);
        let weak = Rc::downgrade(self);
        entry.connect_changed(move |e| {
            let text = e.text().to_string();
            if let Some(app) = weak.upgrade() {
                edit_at(&app, at, move |f| f.label = text);
            }
        });
        entry
    }

    /// Champ du chemin : enregistré à chaque frappe, entouré de rouge tant que ce n'est pas un dossier existant.
    pub fn path_entry(self: &Rc<Self>, at: usize, folder: &SideFolder) -> Entry {
        let entry = Entry::new();
        entry.add_css_class("hex");
        entry.set_text(&folder.path);
        entry.set_hexpand(true);
        entry.set_tooltip_text(Some("Chemin complet du dossier"));
        set_class(&entry, "invalid", !Path::new(&folder.path).is_dir());
        let weak = Rc::downgrade(self);
        entry.connect_changed(move |e| {
            let text = e.text().trim().to_string();
            set_class(e, "invalid", !Path::new(&text).is_dir());
            if let Some(app) = weak.upgrade() {
                edit_at(&app, at, move |f| f.path = text);
            }
        });
        entry
    }
}
