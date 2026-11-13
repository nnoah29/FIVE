//! Contrôles de la page Paramètres : ligne de réglage, groupe segmenté, case à cocher.

use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Button, Widget};

use crate::app::shared::App;
use crate::model::settings::Settings;
use crate::ui::classes::set_class;
use crate::ui::controls::tick;
use crate::ui::widgets::{hbox, label, vbox};

/// Modification d'un réglage.
pub type Change = Rc<dyn Fn(&mut Settings)>;

/// Une option d'un groupe segmenté : libellé, état pressé, modification à appliquer.
pub type Choice = (&'static str, bool, Change);

/// Ligne de réglage : titre et explication à gauche, contrôle à droite (ou dessous si `stack`).
pub fn setting_row(title: &str, text: Option<&str>, control: &impl IsA<Widget>, stack: bool) -> GtkBox {
    let row = if stack { vbox(16) } else { hbox(32) };
    row.add_css_class("setting");
    let column = vbox(0);
    column.set_hexpand(true);
    column.set_valign(Align::Center);
    column.append(&label(title, &["setting-title"]));
    if let Some(text) = text {
        let note = label(text, &["setting-text"]);
        note.set_wrap(true);
        note.set_max_width_chars(62);
        column.append(&note);
    }
    control.set_halign(if stack { Align::Start } else { Align::End });
    control.set_valign(Align::Center);
    row.append(&column);
    row.append(control);
    row
}

/// Groupe de boutons segmentés ; un clic applique la modification et redessine la page.
pub fn choice(app: &Rc<App>, options: Vec<Choice>) -> GtkBox {
    let group = hbox(0);
    group.add_css_class("seg");
    for (text, pressed, change) in options {
        let button = Button::new();
        button.set_child(Some(&label(&text.to_uppercase(), &[])));
        button.add_css_class("seg-btn");
        set_class(&button, "pressed", pressed);
        let weak = Rc::downgrade(app);
        button.connect_clicked(move |_| {
            if let Some(app) = weak.upgrade() {
                let change = change.clone();
                app.change_setting(move |s| change(s));
            }
        });
        group.append(&button);
    }
    group
}

/// Case à cocher liée à un réglage booléen.
pub fn checkbox(app: &Rc<App>, on: bool, set: fn(&mut Settings, bool)) -> Button {
    let button = tick(on);
    let weak = Rc::downgrade(app);
    button.connect_clicked(move |_| {
        if let Some(app) = weak.upgrade() {
            app.change_setting(move |s| set(s, !on));
        }
    });
    button
}
