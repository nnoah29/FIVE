//! Une règle dans la liste : état, nom, résumé, raison d'invalidité, boutons d'action.

use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{Box as GtkBox, FlowBox, Label, SelectionMode};
use serde_json::Value;

use crate::app::shared::{App, on_click};
use crate::model::rules_labels::status_label;
use crate::model::rules_summary::summarize;
use crate::ui::buttons::icon_button;
use crate::ui::widgets::{hbox, label, text_button, vbox};

/// Action d'un bouton de la liste.
type Run = Box<dyn Fn(&Rc<App>)>;

/// Pastille d'état : « En essai », « Active », « Désactivée » ou « Invalide ».
fn badge(status: &str) -> Label {
    label(&status_label(status).to_uppercase(), &["badge", &format!("badge-{status}")])
}

impl App {
    /// Tête de ligne : pastille, nom, monter et descendre.
    fn rule_head(self: &Rc<Self>, at: usize, total: usize, rule: &Value) -> GtkBox {
        let head = hbox(12);
        head.append(&badge(rule["status"].as_str().unwrap_or("invalid")));
        let name = label(rule["name"].as_str().unwrap_or(""), &["setting-title"]);
        name.set_hexpand(true);
        name.set_halign(gtk4::Align::Fill);
        name.set_ellipsize(gtk4::pango::EllipsizeMode::End);
        name.set_max_width_chars(1);
        head.append(&name);
        for (glyph, delta) in [("up", -1isize), ("down", 1)] {
            let button = icon_button(glyph, &["small"]);
            button.set_sensitive(if delta < 0 { at > 0 } else { at + 1 < total });
            let id = rule["id"].as_str().unwrap_or("").to_string();
            on_click(&button, self, move |app| app.rule_move(&id, delta));
            head.append(&button);
        }
        head
    }

    /// Boutons selon l'état : aperçu, activer, désactiver, appliquer, modifier, supprimer.
    fn rule_buttons(self: &Rc<Self>, rule: &Value) -> FlowBox {
        let (id, name, status) =
            (rule["id"].as_str().unwrap_or("").to_string(), rule["name"].as_str().unwrap_or("").to_string(), rule["status"].as_str().unwrap_or(""));
        let flow = FlowBox::new();
        flow.set_selection_mode(SelectionMode::None);
        flow.set_column_spacing(8);
        flow.set_row_spacing(8);
        let add = |text: &str, icon: &str, primary: bool, run: Run| {
            let button = text_button(text, Some(icon), if primary { &["go"] } else { &[] });
            on_click(&button, self, move |app| run(app));
            flow.append(&button);
        };
        let (a, b, c, d, e) = (id.clone(), id.clone(), id.clone(), id.clone(), id.clone());
        if status != "invalid" && status != "disabled" {
            add("Aperçu", "eye", false, Box::new(move |app| app.open_rule_preview(&a)));
        }
        match status {
            "trial" => add("Activer", "check", true, Box::new(move |app| app.rule_enable(&b, true, true))),
            "active" => add("Appliquer maintenant", "play", false, Box::new(move |app| app.rule_run_confirm(&b))),
            "disabled" => add("Réactiver", "play", true, Box::new(move |app| app.rule_enable(&b, true, false))),
            _ => {}
        }
        if status == "trial" || status == "active" {
            add("Désactiver", "pause", false, Box::new(move |app| app.rule_enable(&c, false, false)));
        }
        add("Modifier", "pen", false, Box::new(move |app| app.open_rule_editor(Some(d.clone()))));
        add("Supprimer", "trash2", false, Box::new(move |app| app.rule_delete_confirm(&e, &name)));
        flow
    }

    /// Ligne d'une règle.
    pub fn rule_row(self: &Rc<Self>, at: usize, total: usize, rule: &Value) -> GtkBox {
        let row = vbox(10);
        row.add_css_class("setting");
        row.append(&self.rule_head(at, total, rule));
        let summary = label(&summarize(rule), &["setting-text"]);
        summary.set_wrap(true);
        summary.set_max_width_chars(1);
        summary.set_hexpand(true);
        summary.set_halign(gtk4::Align::Fill);
        row.append(&summary);
        if let Some(reason) = rule["reason"].as_str().filter(|r| !r.is_empty()) {
            let why = label(reason, &["setting-text", "rule-reason"]);
            why.set_wrap(true);
            why.set_max_width_chars(1);
            row.append(&why);
        }
        row.append(&self.rule_buttons(rule));
        row
    }
}
