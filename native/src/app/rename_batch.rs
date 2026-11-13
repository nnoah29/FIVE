//! Renommage par lot : remplacement ou modèle, aperçu « ancien → nouveau » mis à jour en direct.

use std::cell::RefCell;
use std::rc::Rc;

use five_engine::entry::Entry;
use five_engine::rename::rename_many;
use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Button, Label, PolicyType, ScrolledWindow};

use super::shared::{App, on_click};
use super::work::spawn_work;
use crate::model::rename_plan::{RenameSpec, check_names, new_name};
use crate::ui::classes::{clear, set_button_text, set_class};
use crate::ui::icon::icon;
use crate::ui::widgets::{hbox, label, text_button, vbox};

/// État partagé de la fenêtre de renommage par lot.
pub struct Batch {
    pub spec: RefCell<RenameSpec>,
    pub items: Vec<Entry>,
    pub siblings: Vec<String>,
    pub fields: GtkBox,
    pub list: GtkBox,
    pub note: Label,
    pub ok: Button,
    pub plan: RefCell<Vec<(String, String)>>,
}

/// Ligne « ancien → nouveau » de l'aperçu, avec la raison du refus s'il y en a une.
fn rename_line(old: &str, new: &str, error: &str) -> GtkBox {
    let line = hbox(10);
    line.add_css_class("rn-line");
    set_class(&line, "bad", !error.is_empty());
    let before = label(old, &[]);
    before.set_hexpand(true);
    line.append(&before);
    line.append(&icon("back"));
    let after = label(new, &["rn-new"]);
    after.set_hexpand(true);
    line.append(&after);
    if !error.is_empty() {
        line.append(&label(error, &["rn-err"]));
    }
    line
}

impl Batch {
    /// Calcule les nouveaux noms, vérifie les conflits, redessine la liste et le bouton.
    pub fn refresh(self: &Rc<Self>) {
        let olds: Vec<String> = self.items.iter().map(|i| i.name.clone()).collect();
        let spec = self.spec.borrow().clone();
        let mut message = String::new();
        let news: Vec<String> = olds
            .iter()
            .enumerate()
            .map(|(at, name)| {
                new_name(&spec, name, at).unwrap_or_else(|e| {
                    message = format!("Expression invalide : {e}");
                    name.clone()
                })
            })
            .collect();
        self.note.set_text(&message);
        let errors = check_names(&olds, &news, &self.siblings);
        let plan: Vec<(String, String)> =
            (0..olds.len()).filter(|&i| news[i] != olds[i] || !errors[i].is_empty()).map(|i| (self.items[i].path.clone(), news[i].clone())).collect();
        clear(&self.list);
        for at in 0..olds.len() {
            self.list.append(&rename_line(&olds[at], &news[at], &errors[at]));
        }
        self.ok.set_sensitive(!plan.is_empty() && errors.iter().all(String::is_empty) && message.is_empty());
        set_button_text(&self.ok, &if plan.is_empty() { "Aucun changement".to_string() } else { format!("Renommer {} élément(s)", plan.len()) });
        *self.plan.borrow_mut() = plan;
    }
}

/// Construit l'état et les widgets de la fenêtre de renommage dans la modale.
fn build_batch(app: &Rc<App>, modal: &super::modal::Modal, items: Vec<Entry>) -> Rc<Batch> {
    modal.dialog.add_css_class("dialog");
    modal.dialog.append(&label(&format!("RENOMMER {} ÉLÉMENTS", items.len()), &["h2"]));
    let siblings = app.st.borrow().items.iter().map(|i| i.name.clone()).collect();
    let batch = Rc::new(Batch {
        spec: RefCell::new(RenameSpec::default()),
        items,
        siblings,
        fields: vbox(10),
        list: vbox(0),
        note: label("", &["form-error", "mono"]),
        ok: text_button("Renommer", None, &["go"]),
        plan: RefCell::new(Vec::new()),
    });
    batch.list.add_css_class("rn-list");
    let scroll =
        ScrolledWindow::builder().hscrollbar_policy(PolicyType::Never).max_content_height(300).propagate_natural_height(true).child(&batch.list).build();
    let actions = hbox(8);
    actions.set_halign(Align::End);
    let cancel = text_button("Annuler", None, &[]);
    let target = modal.clone();
    on_click(&cancel, app, move |app| app.close_modal(&target));
    actions.append(&cancel);
    actions.append(&batch.ok);
    for widget in [batch.fields.upcast_ref::<gtk4::Widget>(), scroll.upcast_ref(), batch.note.upcast_ref(), actions.upcast_ref()] {
        modal.dialog.append(widget);
    }
    batch
}

/// Bouton « Renommer » : lance le renommage du plan en arrière-plan puis ferme ou affiche l'erreur.
fn wire_apply(app: &Rc<App>, batch: &Rc<Batch>, modal: &super::modal::Modal) {
    let (apply, target) = (batch.clone(), modal.clone());
    on_click(&batch.ok, app, move |app| {
        let (root, pairs, done, modal) = (app.root.clone(), apply.plan.borrow().clone(), app.clone(), target.clone());
        apply.ok.set_sensitive(false);
        let (note, again) = (apply.note.clone(), apply.ok.clone());
        spawn_work(
            move || rename_many(&root, &pairs).map(|_| ()).map_err(|r| r.message),
            move |result| match result {
                Ok(()) => {
                    done.toast("Renommé");
                    done.close_modal(&modal);
                    done.reload();
                }
                Err(message) => {
                    note.set_text(&message);
                    again.set_sensitive(true);
                }
            },
        );
    });
}

impl App {
    /// Ouvre la fenêtre de renommage de plusieurs éléments.
    pub fn batch_rename(self: &Rc<Self>, items: Vec<Entry>) {
        let modal = self.open_modal(720, false);
        let batch = build_batch(self, &modal, items);
        wire_apply(self, &batch, &modal);
        batch.draw_fields();
        batch.refresh();
    }
}
