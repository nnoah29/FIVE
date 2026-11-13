//! Champs de la fenêtre de renommage par lot : mode, saisies, cases à cocher.

use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{Align, Button, CheckButton, Entry as Input};

use super::rename_batch::Batch;
use crate::model::rename_plan::RenameSpec;
use crate::ui::classes::{clear, set_class};
use crate::ui::widgets::hbox;

impl Batch {
    /// Champ de saisie lié à un réglage : `set` écrit la valeur saisie dans le plan.
    pub fn input(self: &Rc<Self>, text: &str, value: &str, set: fn(&mut RenameSpec, &str)) -> Input {
        let entry = crate::app::dialog::field(value);
        entry.set_placeholder_text(Some(text));
        let me = self.clone();
        entry.connect_changed(move |e| {
            set(&mut me.spec.borrow_mut(), &e.text());
            me.refresh();
        });
        entry
    }

    /// Case à cocher liée à un réglage booléen.
    pub fn toggle(self: &Rc<Self>, text: &str, on: bool, set: fn(&mut RenameSpec, bool)) -> CheckButton {
        let check = CheckButton::with_label(text);
        check.set_active(on);
        check.add_css_class("rn-check");
        let me = self.clone();
        check.connect_toggled(move |c| {
            set(&mut me.spec.borrow_mut(), c.is_active());
            me.refresh();
        });
        check
    }

    /// Choix du mode : Remplacer ou Modèle.
    fn mode_switch(self: &Rc<Self>) -> gtk4::Box {
        let modes = hbox(0);
        modes.add_css_class("seg");
        modes.set_halign(Align::Start);
        let template_mode = self.spec.borrow().template_mode;
        for (template, text) in [(false, "Remplacer"), (true, "Modèle")] {
            let button = Button::with_label(&text.to_uppercase());
            button.add_css_class("seg-btn");
            set_class(&button, "pressed", template_mode == template);
            let me = self.clone();
            button.connect_clicked(move |_| {
                me.spec.borrow_mut().template_mode = template;
                me.draw_fields();
                me.refresh();
            });
            modes.append(&button);
        }
        modes
    }

    /// Champs du mode Modèle : modèle, début, nombre de chiffres.
    fn template_fields(self: &Rc<Self>, spec: &RenameSpec) {
        self.fields.append(&self.input("Modèle : {name} {n} {ext}", &spec.template, |s, v| s.template = v.to_string()));
        let numbers = hbox(16);
        numbers.append(&self.input("Début", &spec.start.to_string(), |s, v| s.start = v.parse().unwrap_or(0)));
        numbers.append(&self.input("Chiffres", &spec.pad.to_string(), |s, v| s.pad = v.parse().unwrap_or(0)));
        self.fields.append(&numbers);
    }

    /// Champs du mode Remplacer : recherche, remplacement, options.
    fn replace_fields(self: &Rc<Self>, spec: &RenameSpec) {
        self.fields.append(&self.input("Rechercher", &spec.find, |s, v| s.find = v.to_string()));
        self.fields.append(&self.input("Remplacer par", &spec.replacement, |s, v| s.replacement = v.to_string()));
        let toggles = hbox(16);
        toggles.append(&self.toggle("Expression régulière", spec.regex, |s, v| s.regex = v));
        toggles.append(&self.toggle("Ignorer la casse", spec.ignore_case, |s, v| s.ignore_case = v));
        self.fields.append(&toggles);
    }

    /// Redessine les champs selon le mode choisi.
    pub fn draw_fields(self: &Rc<Self>) {
        clear(&self.fields);
        self.fields.append(&self.mode_switch());
        let spec = self.spec.borrow().clone();
        if spec.template_mode { self.template_fields(&spec) } else { self.replace_fields(&spec) }
    }
}
