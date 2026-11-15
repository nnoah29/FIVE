//! Emplacements de la barre latérale : Dossier personnel, vues virtuelles, corbeille, dossiers XDG.

use std::rc::Rc;

use gtk4::Button;
use gtk4::prelude::*;

use crate::app::shared::{App, on_click};
use crate::state::Place;
use crate::ui::classes::{clear, set_class};
use crate::ui::icon::icon;
use crate::ui::widgets::{hbox, label};

/// Entrée de navigation : identifiant d'icône, libellé, vue virtuelle éventuelle, dossier cible.
struct NavEntry {
    id: String,
    text: String,
    view: Option<String>,
    target: String,
}

/// Bouton d'emplacement : icône et libellé.
pub fn nav_button(icon_name: &str, text: &str, current: bool) -> Button {
    let button = Button::new();
    button.add_css_class("navitem");
    set_class(&button, "current", current);
    let row = hbox(14);
    row.append(&icon(icon_name));
    let name = label(text, &[]);
    name.set_ellipsize(gtk4::pango::EllipsizeMode::End);
    name.set_hexpand(true);
    row.append(&name);
    button.set_child(Some(&row));
    button
}

/// Entrées fixes (Dossier personnel, vues virtuelles) suivies de la corbeille si elle existe.
fn main_entries(places: &[Place]) -> Vec<NavEntry> {
    let fixed = [
        ("home", "Dossier personnel", None),
        ("recent", "Récents", Some("recent")),
        ("favorites", "Favoris", Some("favorites")),
        ("network", "Réseau", Some("network")),
    ];
    let mut entries: Vec<NavEntry> =
        fixed.iter().map(|(id, text, v)| NavEntry { id: id.to_string(), text: text.to_string(), view: v.map(String::from), target: String::new() }).collect();
    entries.extend(places.iter().filter(|p| p.id == "trash").map(|p| NavEntry { id: p.id.clone(), text: p.label.clone(), view: None, target: p.path.clone() }));
    entries
}

/// Indique si l'entrée correspond à l'emplacement courant.
fn is_current(entry: &NavEntry, path: &str, view: &Option<String>, searching: bool) -> bool {
    if let Some(name) = &entry.view {
        return view.as_deref() == Some(name.as_str());
    }
    let inside = path == entry.target || path.starts_with(&format!("{}/", entry.target));
    view.is_none() && !searching && if entry.target.is_empty() { path.is_empty() } else { inside }
}

impl App {
    /// Bouton d'une entrée, câblé vers la vue ou le dossier cible.
    fn nav_row(self: &Rc<Self>, entry: NavEntry, here: bool) -> Button {
        let button = nav_button(&entry.id, &entry.text, here);
        on_click(&button, self, move |app| match &entry.view {
            Some(name) => app.go_view(name),
            None => app.go(&entry.target),
        });
        button
    }

    /// Reconstruit les emplacements, puis la section « Dossiers » (automatique ou choisie dans les paramètres).
    pub fn render_places(self: &Rc<Self>) {
        clear(&self.ui.side.places);
        let (places, path, view, searching) = {
            let st = self.st.borrow();
            (st.places.clone(), st.path.clone(), st.virtual_view.clone(), st.searching())
        };
        let others = self.side_places();
        for entry in main_entries(&places) {
            let here = is_current(&entry, &path, &view, searching);
            self.ui.side.places.append(&self.nav_row(entry, here));
        }
        if !others.is_empty() {
            self.ui.side.places.append(&label("DOSSIERS", &["nav-title"]));
        }
        for place in &others {
            let entry = NavEntry { id: place.id.clone(), text: place.label.clone(), view: None, target: place.path.clone() };
            let here = is_current(&entry, &path, &view, searching);
            self.ui.side.places.append(&self.nav_row(entry, here));
        }
    }
}
