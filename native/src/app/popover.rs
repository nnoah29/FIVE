//! Menus contextuels : fenêtre flottante ancrée à un widget, entrées avec icône et raccourci.

use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{Button, Popover, PositionType, Widget};

use super::menu_place::MenuPlace;

use super::shared::App;
use crate::ui::icon::icon;
use crate::ui::widgets::{hbox, label, vbox};

/// Action d'une entrée de menu.
pub type MenuAction = Rc<dyn Fn(&Rc<App>)>;

/// Une entrée de menu : icône, texte, raccourci affiché, action.
pub struct MenuEntry {
    pub icon: &'static str,
    pub label: String,
    pub hint: Option<&'static str>,
    pub run: MenuAction,
}

impl MenuEntry {
    /// Entrée de menu sans raccourci.
    pub fn new(icon: &'static str, label: &str, run: impl Fn(&Rc<App>) + 'static) -> MenuEntry {
        MenuEntry { icon, label: label.to_string(), hint: None, run: Rc::new(run) }
    }

    /// Ajoute le raccourci affiché à droite.
    pub fn hint(mut self, hint: &'static str) -> MenuEntry {
        self.hint = Some(hint);
        self
    }
}

/// Bouton d'une entrée : icône, texte, raccourci.
fn entry_button(entry: &MenuEntry) -> Button {
    let button = Button::new();
    button.add_css_class("menu-item");
    let row = hbox(12);
    row.append(&icon(entry.icon));
    let text = label(&entry.label, &[]);
    text.set_hexpand(true);
    row.append(&text);
    if let Some(hint) = entry.hint {
        row.append(&label(hint, &["hint"]));
    }
    button.set_child(Some(&row));
    button
}

impl App {
    /// Ferme le menu ouvert, s'il y en a un.
    pub fn close_menu(&self) {
        if let Some(popover) = self.menu.borrow_mut().take() {
            popover.popdown();
        }
    }

    /// Liste de boutons du menu : chaque clic referme le menu puis lance l'action.
    fn menu_list(self: &Rc<Self>, popover: &Popover, entries: Vec<MenuEntry>) -> gtk4::Box {
        let list = vbox(0);
        for entry in entries {
            let button = entry_button(&entry);
            let (weak, run, pop) = (Rc::downgrade(self), entry.run.clone(), popover.clone());
            button.connect_clicked(move |_| {
                pop.popdown();
                if let Some(app) = weak.upgrade() {
                    run(&app);
                }
            });
            list.append(&button);
        }
        list
    }

    /// Ouvre un menu placé selon `place` ; un second appel sur la même ancre le referme.
    pub fn popover_at(self: &Rc<Self>, anchor: &impl IsA<Widget>, place: MenuPlace, entries: Vec<MenuEntry>) {
        let toggled = self.menu.borrow().as_ref().is_some_and(|p| p.parent().as_ref() == Some(anchor.upcast_ref()));
        self.close_menu();
        if toggled {
            return;
        }
        let popover = Popover::new();
        popover.set_child(Some(&self.menu_list(&popover, entries)));
        popover.set_parent(anchor);
        popover.set_has_arrow(false);
        popover.set_position(PositionType::Bottom);
        place.apply(&popover);
        popover.add_css_class("menu");
        popover.connect_closed(|p| {
            let p = p.clone();
            gtk4::glib::idle_add_local_once(move || p.unparent());
        });
        popover.popup();
        *self.menu.borrow_mut() = Some(popover);
    }
}
