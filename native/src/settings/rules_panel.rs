//! Onglet « Règles » : interrupteur général, liste des règles, avis sur le fichier, historique.

use std::rc::Rc;

use five_engine::rules::api_read::list_rules;
use gtk4::prelude::*;
use gtk4::{Align, Widget};
use serde_json::Value;

use super::controls::toggle_row;
use crate::app::shared::{App, on_click};
use crate::ui::widgets::{hbox, label, text_button};

/// Avis affichés en tête : fichier abîmé, règles illisibles, fichier d'une version plus récente.
fn notices(list: &Value) -> Vec<Widget> {
    let notice = |text: &str| {
        let row = hbox(0);
        row.add_css_class("setting");
        let message = label(text, &["setting-text", "rule-reason"]);
        message.set_wrap(true);
        message.set_max_width_chars(1);
        message.set_hexpand(true);
        row.append(&message);
        row.upcast::<Widget>()
    };
    let mut out = Vec::new();
    if list["damaged"] == true {
        out.push(notice("Le fichier des règles était abîmé : il a été mis de côté (regles.json.abime) et la liste est repartie de zéro."));
    }
    if list["newer"] == true {
        out.push(notice("Le fichier des règles vient d'une version plus récente de FIVE : il est conservé tel quel et n'est pas appliqué."));
    }
    if list["unreadable"].as_u64().unwrap_or(0) > 0 {
        out.push(notice(&format!("{} règle(s) illisible(s) conservée(s) telles quelles dans le fichier.", list["unreadable"])));
    }
    out
}

impl App {
    /// Ligne du bas : créer une règle.
    fn rules_footer(self: &Rc<Self>) -> Widget {
        let row = hbox(12);
        row.add_css_class("setting");
        row.set_halign(Align::Fill);
        let new = text_button("Nouvelle règle", Some("plus"), &["go"]);
        on_click(&new, self, |app| app.open_rule_editor(None));
        row.append(&new);
        row.upcast()
    }

    /// Onglet « Règles » : interrupteur, avis, une ligne par règle, bouton de création, historique.
    pub fn rules_panel(self: &Rc<Self>) -> Vec<Widget> {
        let list = list_rules(&self.root, &self.rules_dir());
        let rules = list["rules"].as_array().cloned().unwrap_or_default();
        let text = "FIVE surveille les dossiers de vos règles toutes les 5 secondes tant qu'il est ouvert. Une règle neuve reste « en essai » : elle journalise ce qu'elle ferait, sans rien modifier.";
        let mut rows = vec![toggle_row(self, "Appliquer les règles automatiquement", text, self.set.borrow().rules_enabled, |s, v| s.rules_enabled = v)];
        rows.extend(notices(&list));
        rows.extend(rules.iter().enumerate().map(|(at, rule)| self.rule_row(at, rules.len(), rule).upcast::<Widget>()));
        rows.push(self.rules_footer());
        rows.extend(self.history_rows(&rules));
        rows
    }
}
