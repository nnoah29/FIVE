//! Aperçu rapide : l'élément en grand, parcours aux flèches, fermeture à Espace ou Échap.

use std::cell::Cell;
use std::rc::Rc;

use five_engine::entry::Entry;
use gtk4::gdk::Key;
use gtk4::glib::Propagation;
use gtk4::prelude::*;
use gtk4::{Box as GtkBox, EventControllerKey, Label};

use super::modal::Modal;
use super::shared::{App, on_click};
use crate::model::kinds::kind_of;
use crate::ui::buttons::icon_button;
use crate::ui::classes::clear;
use crate::ui::widgets::{hbox, label, mono, vbox};

/// Éléments visibles pendant l'aperçu et position courante.
struct Look {
    list: Vec<Entry>,
    index: Cell<usize>,
}

impl App {
    /// Affiche l'élément courant de l'aperçu rapide dans la zone et le titre.
    fn look_show(self: &Rc<Self>, look: &Look, title: &Label, zone: &GtkBox) {
        let item = &look.list[look.index.get()];
        title.set_text(&item.name.to_uppercase());
        clear(zone);
        zone.remove_css_class("none");
        let height = ((self.ui.window.height() as f64 * 0.88).min(760.0) - 110.0) as i32;
        self.fill_preview(zone, item, kind_of(item), None, height);
    }

    /// Branche les touches de l'aperçu : flèches pour parcourir, Espace et Échap pour fermer.
    fn look_keys(self: &Rc<Self>, modal: &Modal, look: Rc<Look>, title: Label, zone: GtkBox) {
        let keys = EventControllerKey::new();
        let (weak, modal2) = (Rc::downgrade(self), modal.clone());
        keys.connect_key_pressed(move |_, key, _, _| {
            let Some(app) = weak.upgrade() else { return Propagation::Proceed };
            let step: isize = match key {
                Key::Right | Key::Down => 1,
                Key::Left | Key::Up => -1,
                Key::Escape | Key::space => {
                    app.close_modal(&modal2);
                    app.select(look.index.get() as isize, gtk4::gdk::ModifierType::empty());
                    return Propagation::Stop;
                }
                _ => return Propagation::Proceed,
            };
            look.index.set((look.index.get() as isize + step).clamp(0, look.list.len() as isize - 1) as usize);
            app.look_show(&look, &title, &zone);
            Propagation::Stop
        });
        modal.layer.add_controller(keys);
        modal.layer.set_focusable(true);
        modal.layer.grab_focus();
    }

    /// Position de départ de l'aperçu : l'élément demandé, ou le premier élément sélectionné.
    fn look_start(&self, item: &Option<Entry>, list: &[Entry]) -> Option<usize> {
        let picked = self.st.borrow().picked.clone();
        match item {
            Some(wanted) => list.iter().position(|i| i.path == wanted.path),
            None => list.iter().position(|i| picked.contains(&i.path)),
        }
    }

    /// Ouvre l'aperçu rapide sur `item`, ou sur la sélection courante si `item` est absent.
    pub fn quick_look(self: &Rc<Self>, item: Option<Entry>) {
        if self.modal_open() {
            return;
        }
        let list = self.st.borrow().shown.clone();
        let Some(start) = self.look_start(&item, &list) else { return self.toast("Sélectionnez un élément") };
        let modal = self.open_modal((self.ui.window.width() as f64 * 0.94).min(1100.0) as i32, false);
        modal.dialog.add_css_class("ql");
        let (title, zone) = (label("", &["h2"]), vbox(8));
        title.set_hexpand(true);
        title.set_halign(gtk4::Align::Fill);
        title.set_ellipsize(gtk4::pango::EllipsizeMode::End);
        zone.add_css_class("preview");
        zone.set_vexpand(true);
        modal.dialog.set_size_request(modal.dialog.width_request(), (self.ui.window.height() as f64 * 0.88).min(760.0) as i32);
        let head: GtkBox = hbox(16);
        let close = icon_button("x", &[]);
        let target = modal.clone();
        on_click(&close, self, move |app| app.close_modal(&target));
        for widget in [title.upcast_ref::<gtk4::Widget>(), mono("← → parcourir · Espace fermer", &["dim"]).upcast_ref(), close.upcast_ref()] {
            head.append(widget);
        }
        modal.dialog.append(&head);
        modal.dialog.append(&zone);
        let look = Rc::new(Look { list, index: Cell::new(start) });
        self.look_show(&look, &title, &zone);
        self.look_keys(&modal, look, title, zone);
    }
}
