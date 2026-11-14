//! Historique des règles : dernières actions journalisées et annulation de la dernière exécution.

use std::collections::HashMap;
use std::rc::Rc;

use five_engine::rules::api_read::rules_history;
use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Widget};
use serde_json::Value;

use crate::app::shared::{App, on_click};
use crate::model::rules_labels::{action_label, outcome_label};
use crate::ui::widgets::{hbox, label, text_button, vbox};

/// `2026-11-14T10:03:00` devient `14/11 10:03`.
fn when(stamp: &str) -> String {
    let (date, time) = stamp.split_once('T').unwrap_or((stamp, ""));
    let mut parts = date.split('-').skip(1);
    let (month, day) = (parts.next().unwrap_or(""), parts.next().unwrap_or(""));
    format!("{day}/{month} {}", time.get(..5).unwrap_or(""))
}

/// Ligne d'historique : date, règle, action, fichier, issue ; une ligne d'annulation est résumée.
fn history_line(entry: &Value, names: &HashMap<String, String>) -> GtkBox {
    let line = hbox(12);
    line.add_css_class("hist-line");
    line.append(&label(&when(entry["t"].as_str().unwrap_or("")), &["mono", "dim"]));
    if entry["undo_of"].is_string() {
        line.append(&label("Exécution annulée", &["setting-text"]));
        return line;
    }
    let rule = names.get(entry["rule"].as_str().unwrap_or("")).cloned().unwrap_or_else(|| "règle supprimée".into());
    let what = format!("{} · {}", rule, action_label(entry["action"].as_str().unwrap_or("")));
    let title = label(&what, &["setting-text"]);
    title.set_hexpand(true);
    title.set_halign(Align::Fill);
    title.set_ellipsize(gtk4::pango::EllipsizeMode::End);
    title.set_max_width_chars(1);
    let path = label(entry["path"].as_str().unwrap_or(""), &["mono", "dim"]);
    path.set_ellipsize(gtk4::pango::EllipsizeMode::Start);
    path.set_width_chars(28);
    path.set_max_width_chars(28);
    let outcome = entry["outcome"].as_str().unwrap_or("");
    for widget in [title.upcast::<Widget>(), path.upcast(), label(&outcome_label(outcome).to_uppercase(), &["badge", &format!("badge-{outcome}")]).upcast()] {
        line.append(&widget);
    }
    line
}

impl App {
    /// En-tête de l'historique : titre et bouton d'annulation.
    fn history_head(self: &Rc<Self>) -> GtkBox {
        let head = hbox(12);
        let title = label("Historique", &["setting-title"]);
        title.set_hexpand(true);
        head.append(&title);
        let undo = text_button("Annuler la dernière exécution", Some("back"), &[]);
        on_click(&undo, self, |app| app.rules_undo());
        undo.set_halign(Align::End);
        head.append(&undo);
        head
    }

    /// Bloc d'historique : les 15 dernières lignes du journal, la plus récente d'abord.
    pub fn history_rows(self: &Rc<Self>, rules: &[Value]) -> Vec<Widget> {
        let names: HashMap<String, String> = rules.iter().filter_map(|r| Some((r["id"].as_str()?.to_string(), r["name"].as_str()?.to_string()))).collect();
        let items = rules_history(&self.rules_dir(), 15)["items"].as_array().cloned().unwrap_or_default();
        let block = vbox(10);
        block.add_css_class("setting");
        block.append(&self.history_head());
        if items.is_empty() {
            block.append(&label("Aucune action pour l'instant : l'historique se remplit dès qu'une règle agit ou fait un essai.", &["setting-text"]));
        }
        items.iter().for_each(|entry| block.append(&history_line(entry, &names)));
        vec![block.upcast()]
    }
}
