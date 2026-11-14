//! Clics sur une ligne ou une tuile : sélection, ouverture, menu.

use std::rc::Rc;

use five_engine::entry::Entry;
use gtk4::prelude::*;
use gtk4::{GestureClick, Widget};

use crate::app::shared::App;

/// Câble un clic sur une ligne ou une tuile : simple = sélection, double = ouverture, droit = menu.
pub fn attach_press(app: &Rc<App>, widget: &impl IsA<Widget>, item: &Entry, index: usize) {
    let click = GestureClick::new();
    click.set_button(0);
    let (weak, item, anchor) = (Rc::downgrade(app), item.clone(), widget.clone().upcast::<Widget>());
    click.connect_pressed(move |gesture, presses, _, _| {
        let Some(app) = weak.upgrade() else { return };
        let mods = gesture.current_event_state();
        match (gesture.current_button(), presses) {
            (3, _) => app.open_menu_at(&anchor, &item, Some(index)),
            (1, 2) => app.open_index(index),
            (1, _) => app.select(index as isize, mods),
            _ => {}
        }
    });
    widget.add_controller(click);
}
