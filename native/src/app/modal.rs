//! Fenêtres modales : couche sombre sur toute la fenêtre et boîte centrée.

use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, GestureClick, Widget};

use super::shared::App;
use crate::ui::widgets::vbox;

/// Une fenêtre modale ouverte : sa couche et sa boîte de contenu.
#[derive(Clone)]
pub struct Modal {
    pub layer: GtkBox,
    pub dialog: GtkBox,
    previous_focus: Option<Widget>,
}

impl App {
    /// Ouvre une modale ; `top` l'aligne en haut (palette), `width` fixe sa largeur en pixels.
    pub fn open_modal(self: &Rc<Self>, width: i32, top: bool) -> Modal {
        let dialog = vbox(16);
        dialog.set_size_request(width, -1);
        dialog.set_halign(Align::Center);
        dialog.set_valign(if top { Align::Start } else { Align::Center });
        dialog.set_vexpand(!top);
        if top {
            dialog.set_margin_top(90);
        }
        let layer = GtkBox::new(gtk4::Orientation::Vertical, 0);
        layer.add_css_class("overlay-dim");
        layer.set_hexpand(true);
        layer.set_vexpand(true);
        layer.append(&dialog);
        let click = GestureClick::new();
        let (weak, backdrop) = (Rc::downgrade(self), layer.clone());
        click.connect_pressed(move |_, _, x, y| {
            let outside = backdrop.pick(x, y, gtk4::PickFlags::DEFAULT).is_some_and(|w| w == backdrop.clone().upcast::<Widget>());
            if let (true, Some(app)) = (outside, weak.upgrade()) {
                app.close_top_modal();
            }
        });
        layer.add_controller(click);
        self.ui.overlay.add_overlay(&layer);
        let modal = Modal { layer, dialog, previous_focus: GtkWindowExt::focus(&self.ui.window) };
        self.modals.borrow_mut().push(modal.clone());
        modal
    }

    /// Ferme la modale du dessus et rend le focus à l'élément précédent.
    pub fn close_top_modal(self: &Rc<Self>) {
        let top = self.modals.borrow_mut().pop();
        if let Some(modal) = top {
            self.ui.overlay.remove_overlay(&modal.layer);
            if let Some(widget) = modal.previous_focus {
                widget.grab_focus();
            }
        }
    }

    /// Ferme une modale précise, où qu'elle soit dans la pile.
    pub fn close_modal(self: &Rc<Self>, target: &Modal) {
        let position = self.modals.borrow().iter().position(|m| m.layer == target.layer);
        if let Some(at) = position {
            let modal = self.modals.borrow_mut().remove(at);
            self.ui.overlay.remove_overlay(&modal.layer);
        }
    }

    /// Indique si une modale est ouverte.
    pub fn modal_open(&self) -> bool {
        !self.modals.borrow().is_empty()
    }
}
