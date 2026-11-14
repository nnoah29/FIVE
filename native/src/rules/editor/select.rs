//! Sélecteur : un bouton qui ouvre un menu de choix (familles, types, comparaisons, modes…).

use std::rc::Rc;

use gtk4::Button;
use gtk4::prelude::*;

use super::state::Editor;
use crate::app::popover::MenuEntry;
use crate::ui::widgets::text_button;

/// Option d'un sélecteur : valeur technique et libellé.
pub type Choice = (String, String);

/// Bouton affichant le choix courant ; un clic ouvre la liste, un choix appelle `pick` avec sa valeur.
pub fn select_button(ed: &Rc<Editor>, current: &str, options: Vec<Choice>, pick: impl Fn(&Rc<Editor>, String) + 'static) -> Button {
    let shown = options.iter().find(|(value, _)| value == current).map_or_else(|| "Choisir…".to_string(), |(_, text)| text.clone());
    let button = text_button(&shown, None, &["select"]);
    let (weak, pick, current) = (Rc::downgrade(ed), Rc::new(pick), current.to_string());
    let anchor = button.clone();
    button.connect_clicked(move |_| {
        let Some(ed) = weak.upgrade() else { return };
        let Some(app) = ed.app() else { return };
        let entries: Vec<MenuEntry> = options
            .iter()
            .map(|(value, text)| {
                let (picked, run, editor) = (value.clone(), pick.clone(), ed.clone());
                MenuEntry::new(if *value == current { "check" } else { "" }, text, move |_| run(&editor, picked.clone()))
            })
            .collect();
        app.popover(&anchor, entries);
    });
    button
}
