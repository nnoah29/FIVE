//! Boîte de dialogue standard : titre, texte, liste, champ de saisie, erreur, boutons.

use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{Align, Entry, PolicyType, ScrolledWindow};

use super::shared::{on_click, App};
use crate::ui::widgets::{hbox, label, text_button, vbox};

/// Comportement de validation : reçoit la saisie et un rapporteur à appeler avec le résultat.
pub type Submit = Rc<dyn Fn(&Rc<App>, Option<String>, Box<dyn FnOnce(Result<(), String>)>)>;

/// Description d'une boîte de dialogue.
pub struct DialogSpec {
    pub title: String,
    pub text: Option<String>,
    pub names: Option<Vec<String>>,
    pub input: Option<String>,
    pub confirm: String,
    pub submit: Submit,
}

/// Champ de saisie du style de l'interface.
pub fn field(value: &str) -> Entry {
    let entry = Entry::new();
    entry.set_text(value);
    entry.set_max_length(255);
    entry.add_css_class("field");
    entry
}

/// Liste défilante de noms (lignes à chasse fixe).
fn names_list(names: &[String]) -> ScrolledWindow {
    let list = vbox(0);
    list.add_css_class("dialog-list");
    names.iter().for_each(|n| list.append(&label(n, &["dialog-li"])));
    ScrolledWindow::builder().hscrollbar_policy(PolicyType::Never).max_content_height(140).propagate_natural_height(true).child(&list).build()
}

impl App {
    /// Ouvre la boîte de dialogue décrite par `spec`.
    pub fn dialog(self: &Rc<Self>, spec: DialogSpec) {
        let modal = self.open_modal(440, false);
        let box_ = &modal.dialog;
        box_.add_css_class("dialog");
        box_.append(&label(&spec.title.to_uppercase(), &["h2"]));
        if let Some(text) = &spec.text {
            let paragraph = label(text, &["dialog-text"]);
            paragraph.set_wrap(true);
            box_.append(&paragraph);
        }
        if let Some(names) = &spec.names {
            box_.append(&names_list(names));
        }
        let input = spec.input.as_deref().map(field);
        if let Some(entry) = &input {
            box_.append(entry);
            entry.grab_focus();
        }
        let error = label("", &["form-error", "mono"]);
        box_.append(&error);
        let (cancel, ok) = (text_button("Annuler", None, &[]), text_button(&spec.confirm, None, &["go"]));
        let actions = hbox(8);
        actions.set_halign(Align::End);
        actions.append(&cancel);
        actions.append(&ok);
        box_.append(&actions);
        let run = self.dialog_runner(&modal, spec.submit.clone(), input.clone(), error, ok.clone());
        on_click(&cancel, self, {
            let modal = modal.clone();
            move |app| app.close_modal(&modal)
        });
        let again = run.clone();
        ok.connect_clicked(move |_| again());
        if let Some(entry) = &input {
            entry.connect_activate(move |_| run());
        }
    }

    /// Fabrique l'action de validation : désactive le bouton, lance `submit`, ferme ou affiche l'erreur.
    fn dialog_runner(self: &Rc<Self>, modal: &super::modal::Modal, submit: Submit, input: Option<Entry>, error: gtk4::Label, ok: gtk4::Button) -> Rc<dyn Fn()> {
        let (weak, modal) = (Rc::downgrade(self), modal.clone());
        Rc::new(move || {
            let Some(app) = weak.upgrade() else { return };
            ok.set_sensitive(false);
            error.set_text("");
            let (app2, modal2, error2, ok2) = (app.clone(), modal.clone(), error.clone(), ok.clone());
            let report: Box<dyn FnOnce(Result<(), String>)> = Box::new(move |result| match result {
                Ok(()) => app2.close_modal(&modal2),
                Err(message) => {
                    error2.set_text(&message);
                    ok2.set_sensitive(true);
                }
            });
            submit(&app, input.as_ref().map(|e| e.text().to_string()), report);
        })
    }
}
