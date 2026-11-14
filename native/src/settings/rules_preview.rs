//! Aperçu d'une règle : ce qu'elle ferait, fichier par fichier, sans rien modifier.

use std::rc::Rc;
use std::sync::Arc;

use five_engine::rules::api_run::preview_rule;
use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Label, PolicyType, ScrolledWindow};
use serde_json::{Value, json};

use crate::app::modal::Modal;
use crate::app::shared::{App, on_click};
use crate::app::work::spawn_work;
use crate::model::rules_labels::action_label;
use crate::ui::widgets::{hbox, label, text_button, vbox};

/// Ligne de l'aperçu : fichier, action, résultat prévu.
fn preview_line(item: &Value) -> GtkBox {
    let line = hbox(12);
    line.add_css_class("hist-line");
    let path = label(item["path"].as_str().unwrap_or(""), &["mono", "dim"]);
    path.set_ellipsize(gtk4::pango::EllipsizeMode::Start);
    path.set_width_chars(30);
    path.set_max_width_chars(30);
    let text = item["text"].as_str().filter(|t| !t.is_empty()).map_or(String::new(), |t| format!(" → {t}"));
    let what = label(&format!("{}{text}", action_label(item["action"].as_str().unwrap_or(""))), &["setting-text"]);
    what.set_hexpand(true);
    what.set_halign(Align::Fill);
    what.set_ellipsize(gtk4::pango::EllipsizeMode::End);
    what.set_max_width_chars(1);
    line.append(&path);
    line.append(&what);
    line
}

/// Remplit la fenêtre avec le résultat : nombre de fichiers, liste, ou message d'erreur.
fn fill(status: &Label, list: &GtkBox, result: Result<Value, String>) {
    match result {
        Ok(answer) => {
            let count = answer["count"].as_u64().unwrap_or(0);
            let tail = if answer["truncated"] == true { " (500 premières lignes)" } else { "" };
            status.set_text(&if count == 0 {
                "Aucun fichier ne correspond pour l'instant.".to_string()
            } else {
                format!("{count} action(s) prévue(s){tail}. Rien n'est modifié.")
            });
            answer["items"].as_array().into_iter().flatten().for_each(|item| list.append(&preview_line(item)));
        }
        Err(message) => status.set_text(&message),
    }
}

impl App {
    /// Aperçu d'une règle enregistrée (le bouton « Activer » y est proposé si elle est en essai).
    pub fn open_rule_preview(self: &Rc<Self>, id: &str) {
        self.open_preview_of(json!({"id": id}), Some(id.to_string()));
    }

    /// Boutons de l'aperçu : fermer, et activer si la règle est enregistrée.
    fn preview_buttons(self: &Rc<Self>, modal: &Modal, saved: Option<String>) -> GtkBox {
        let actions = hbox(8);
        actions.set_halign(Align::End);
        let close = text_button("Fermer", None, &[]);
        let target = modal.clone();
        on_click(&close, self, move |app| app.close_modal(&target));
        actions.append(&close);
        if let Some(id) = saved {
            let activate = text_button("Activer cette règle", Some("check"), &["go"]);
            let target = modal.clone();
            on_click(&activate, self, move |app| {
                app.close_modal(&target);
                app.rule_enable(&id, true, true);
            });
            actions.append(&activate);
        }
        actions
    }

    /// Corps de la fenêtre d'aperçu : titre, état, liste défilante, boutons.
    fn preview_window(self: &Rc<Self>, saved: Option<String>) -> (Modal, Label, GtkBox) {
        let modal = self.open_modal(760, false);
        modal.dialog.add_css_class("dialog");
        modal.dialog.append(&label("APERÇU DE LA RÈGLE", &["h2"]));
        let status = label("Calcul en cours…", &["dialog-text"]);
        modal.dialog.append(&status);
        let list = vbox(0);
        list.add_css_class("dialog-list");
        let scroll = ScrolledWindow::builder()
            .hscrollbar_policy(PolicyType::Never)
            .min_content_height(120)
            .max_content_height(340)
            .propagate_natural_height(true)
            .child(&list)
            .build();
        modal.dialog.append(&scroll);
        let actions = self.preview_buttons(&modal, saved);
        modal.dialog.append(&actions);
        (modal, status, list)
    }

    /// Calcule l'aperçu de `body` (règle enregistrée ou brouillon) dans un fil de travail et l'affiche.
    pub fn open_preview_of(self: &Rc<Self>, body: Value, saved: Option<String>) {
        let (_modal, status, list) = self.preview_window(saved);
        let (root, jobs, dir) = (self.root.clone(), Arc::clone(&self.jobs), self.rules_dir());
        spawn_work(move || preview_rule(&root, &jobs, &dir, &body).map_err(|r| r.message), move |result| fill(&status, &list, result));
    }
}
