//! Pied du sélecteur : filtre, nom du fichier à enregistrer, boutons Annuler et valider.

use gtk4::prelude::*;
use gtk4::{Button, DropDown, Entry};

use super::request::{Mode, Request};
use crate::ui::widgets::{hbox, text_button};

/// Éléments du pied que la fenêtre doit câbler.
pub struct Footer {
    pub root: gtk4::Box,
    pub name: Entry,
    pub filter: Option<DropDown>,
    pub cancel: Button,
    pub accept: Button,
}

/// Libellé du bouton de validation : celui de l'application, sinon selon le mode.
fn accept_text(request: &Request) -> String {
    let default = match request.mode {
        Mode::Open => "Ouvrir",
        Mode::Folder => "Choisir",
        Mode::Save => "Enregistrer",
    };
    request.accept_label.clone().unwrap_or_else(|| default.to_string())
}

/// Menu des filtres, présent seulement quand l'application en propose.
fn filter_menu(request: &Request) -> Option<DropDown> {
    if request.filters.is_empty() {
        return None;
    }
    let names: Vec<&str> = request.filters.iter().map(|f| f.name.as_str()).collect();
    let menu = DropDown::from_strings(&names);
    menu.set_selected(request.current_filter as u32);
    Some(menu)
}

/// Champ du nom de fichier, visible seulement pour « Enregistrer ».
fn name_entry(request: &Request) -> Entry {
    let entry = Entry::new();
    entry.set_text(&request.current_name);
    entry.set_hexpand(true);
    entry.set_visible(request.mode == Mode::Save);
    entry.set_placeholder_text(Some("Nom du fichier"));
    entry
}

/// Construit le pied : nom, filtre, puis les deux boutons à droite.
pub fn build_footer(request: &Request) -> Footer {
    let root = hbox(10);
    root.add_css_class("chooser-footer");
    let (name, filter) = (name_entry(request), filter_menu(request));
    let (cancel, accept) = (text_button("Annuler", None, &[]), text_button(&accept_text(request), None, &["go"]));
    root.append(&name);
    if let Some(menu) = &filter {
        menu.set_hexpand(request.mode != Mode::Save);
        root.append(menu);
    }
    root.append(&cancel);
    root.append(&accept);
    Footer { root, name, filter, cancel, accept }
}
