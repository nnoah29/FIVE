//! Renommage par lot : remplacement ou modèle, aperçu « ancien → nouveau » mis à jour en direct.

use std::cell::RefCell;
use std::rc::Rc;

use five_engine::entry::Entry;
use five_engine::rename::rename_many;
use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Button, CheckButton, Entry as Input, Label, PolicyType, ScrolledWindow};

use super::shared::{on_click, App};
use super::work::spawn_work;
use crate::model::rename_plan::{check_names, new_name, RenameSpec};
use crate::ui::classes::{clear, set_button_text, set_class};
use crate::ui::icon::icon;
use crate::ui::widgets::{hbox, label, text_button, vbox};

/// État partagé de la fenêtre de renommage par lot.
struct Batch {
    spec: RefCell<RenameSpec>,
    items: Vec<Entry>,
    siblings: Vec<String>,
    fields: GtkBox,
    list: GtkBox,
    note: Label,
    ok: Button,
    plan: RefCell<Vec<(String, String)>>,
}

impl Batch {
    /// Champ de saisie lié à un réglage : `set` écrit la valeur saisie dans le plan.
    fn input(self: &Rc<Self>, text: &str, value: &str, set: fn(&mut RenameSpec, &str)) -> Input {
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
    fn toggle(self: &Rc<Self>, text: &str, on: bool, set: fn(&mut RenameSpec, bool)) -> CheckButton {
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

    /// Redessine les champs selon le mode : remplacer ou modèle.
    fn draw_fields(self: &Rc<Self>) {
        clear(&self.fields);
        let modes = hbox(0);
        modes.add_css_class("seg");
        modes.set_halign(Align::Start);
        let current = self.spec.borrow().clone();
        for (template, text) in [(false, "Remplacer"), (true, "Modèle")] {
            let button = Button::with_label(&text.to_uppercase());
            button.add_css_class("seg-btn");
            set_class(&button, "pressed", current.template_mode == template);
            let me = self.clone();
            button.connect_clicked(move |_| {
                me.spec.borrow_mut().template_mode = template;
                me.draw_fields();
                me.refresh();
            });
            modes.append(&button);
        }
        self.fields.append(&modes);
        if current.template_mode {
            self.fields.append(&self.input("Modèle : {name} {n} {ext}", &current.template, |s, v| s.template = v.to_string()));
            let numbers = hbox(16);
            numbers.append(&self.input("Début", &current.start.to_string(), |s, v| s.start = v.parse().unwrap_or(0)));
            numbers.append(&self.input("Chiffres", &current.pad.to_string(), |s, v| s.pad = v.parse().unwrap_or(0)));
            self.fields.append(&numbers);
        } else {
            self.fields.append(&self.input("Rechercher", &current.find, |s, v| s.find = v.to_string()));
            self.fields.append(&self.input("Remplacer par", &current.replacement, |s, v| s.replacement = v.to_string()));
            let toggles = hbox(16);
            toggles.append(&self.toggle("Expression régulière", current.regex, |s, v| s.regex = v));
            toggles.append(&self.toggle("Ignorer la casse", current.ignore_case, |s, v| s.ignore_case = v));
            self.fields.append(&toggles);
        }
    }

    /// Calcule les nouveaux noms, vérifie les conflits, redessine la liste et le bouton.
    fn refresh(self: &Rc<Self>) {
        let olds: Vec<String> = self.items.iter().map(|i| i.name.clone()).collect();
        let spec = self.spec.borrow().clone();
        let mut message = String::new();
        let news: Vec<String> = olds.iter().enumerate().map(|(at, name)| new_name(&spec, name, at).unwrap_or_else(|e| { message = format!("Expression invalide : {e}"); name.clone() })).collect();
        self.note.set_text(&message);
        let errors = check_names(&olds, &news, &self.siblings);
        let plan: Vec<(String, String)> = (0..olds.len()).filter(|&i| news[i] != olds[i] || !errors[i].is_empty()).map(|i| (self.items[i].path.clone(), news[i].clone())).collect();
        clear(&self.list);
        for at in 0..olds.len() {
            let line = hbox(10);
            line.add_css_class("rn-line");
            set_class(&line, "bad", !errors[at].is_empty());
            let old = label(&olds[at], &[]);
            old.set_hexpand(true);
            line.append(&old);
            line.append(&icon("back"));
            let new = label(&news[at], &["rn-new"]);
            new.set_hexpand(true);
            line.append(&new);
            if !errors[at].is_empty() {
                line.append(&label(&errors[at], &["rn-err"]));
            }
            self.list.append(&line);
        }
        self.ok.set_sensitive(!plan.is_empty() && errors.iter().all(String::is_empty) && message.is_empty());
        set_button_text(&self.ok, &if plan.is_empty() { "Aucun changement".to_string() } else { format!("Renommer {} élément(s)", plan.len()) });
        *self.plan.borrow_mut() = plan;
    }
}

impl App {
    /// Ouvre la fenêtre de renommage de plusieurs éléments.
    pub fn batch_rename(self: &Rc<Self>, items: Vec<Entry>) {
        let modal = self.open_modal(720, false);
        modal.dialog.add_css_class("dialog");
        modal.dialog.append(&label(&format!("RENOMMER {} ÉLÉMENTS", items.len()), &["h2"]));
        let siblings = self.st.borrow().items.iter().map(|i| i.name.clone()).collect();
        let batch = Rc::new(Batch { spec: RefCell::new(RenameSpec::default()), items, siblings, fields: vbox(10), list: vbox(0), note: label("", &["form-error", "mono"]), ok: text_button("Renommer", None, &["go"]), plan: RefCell::new(Vec::new()) });
        batch.list.add_css_class("rn-list");
        let scroll = ScrolledWindow::builder().hscrollbar_policy(PolicyType::Never).max_content_height(300).propagate_natural_height(true).child(&batch.list).build();
        let actions = hbox(8);
        actions.set_halign(Align::End);
        let cancel = text_button("Annuler", None, &[]);
        let target = modal.clone();
        on_click(&cancel, self, move |app| app.close_modal(&target));
        actions.append(&cancel);
        actions.append(&batch.ok);
        for widget in [batch.fields.upcast_ref::<gtk4::Widget>(), scroll.upcast_ref(), batch.note.upcast_ref(), actions.upcast_ref()] {
            modal.dialog.append(widget);
        }
        let (apply, target) = (batch.clone(), modal.clone());
        on_click(&batch.ok, self, move |app| {
            let (root, pairs, done, modal) = (app.root.clone(), apply.plan.borrow().clone(), app.clone(), target.clone());
            apply.ok.set_sensitive(false);
            let note = apply.note.clone();
            let again = apply.ok.clone();
            spawn_work(move || rename_many(&root, &pairs).map(|_| ()).map_err(|r| r.message), move |result| match result {
                Ok(()) => {
                    done.toast("Renommé");
                    done.close_modal(&modal);
                    done.reload();
                }
                Err(message) => {
                    note.set_text(&message);
                    again.set_sensitive(true);
                }
            });
        });
        batch.draw_fields();
        batch.refresh();
    }
}
