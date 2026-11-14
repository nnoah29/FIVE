//! Actions de la liste des règles : activer, réordonner, supprimer, appliquer maintenant, annuler.

use std::rc::Rc;
use std::sync::Arc;

use five_engine::rules::api_read::list_rules;
use five_engine::rules::api_run::{run_rule, undo_rules};
use five_engine::rules::api_write::{delete_rule, enable_rule, reorder_rules};
use serde_json::json;

use crate::app::dialog::DialogSpec;
use crate::app::shared::App;
use crate::app::work::spawn_work;

impl App {
    /// Active (`activate` : sort de l'essai, après aperçu), réactive ou désactive une règle.
    pub fn rule_enable(self: &Rc<Self>, id: &str, enabled: bool, activate: bool) {
        match enable_rule(&self.root, &self.rules_dir(), id, enabled, activate) {
            Ok(_) => self.render_settings(),
            Err(refus) => self.toast(&refus.message),
        }
    }

    /// Décale une règle d'un rang dans l'ordre d'évaluation.
    pub fn rule_move(self: &Rc<Self>, id: &str, delta: isize) {
        let list = list_rules(&self.root, &self.rules_dir());
        let mut ids: Vec<String> = list["rules"].as_array().map(|a| a.iter().filter_map(|r| r["id"].as_str().map(String::from)).collect()).unwrap_or_default();
        let Some(at) = ids.iter().position(|i| i == id) else { return };
        let to = (at as isize + delta).clamp(0, ids.len() as isize - 1) as usize;
        ids.swap(at, to);
        match reorder_rules(&self.rules_dir(), &ids) {
            Ok(_) => self.render_settings(),
            Err(refus) => self.toast(&refus.message),
        }
    }

    /// Demande confirmation, puis supprime la règle.
    pub fn rule_delete_confirm(self: &Rc<Self>, id: &str, name: &str) {
        let id = id.to_string();
        let submit: crate::app::dialog::Submit = Rc::new(move |app, _, report| {
            let result = delete_rule(&app.rules_dir(), &id).map(|_| ()).map_err(|r| r.message);
            if result.is_ok() {
                app.render_settings();
            }
            report(result);
        });
        self.dialog(DialogSpec {
            title: "Supprimer la règle ?".into(),
            text: Some(format!("« {name} » sera supprimée. Ses fichiers déjà traités ne changent pas.")),
            names: None,
            input: None,
            confirm: "Supprimer".into(),
            submit,
        });
    }

    /// Demande confirmation, puis applique la règle à tout son dossier surveillé.
    pub fn rule_run_confirm(self: &Rc<Self>, id: &str) {
        let id = id.to_string();
        let submit: crate::app::dialog::Submit = Rc::new(move |app, _, report| {
            app.rule_run(&id);
            report(Ok(()));
        });
        let text = "La règle sera appliquée à tous les fichiers de son dossier surveillé. Vous pourrez annuler cette exécution depuis l'historique.";
        self.dialog(DialogSpec {
            title: "Appliquer maintenant ?".into(),
            text: Some(text.into()),
            names: None,
            input: None,
            confirm: "Appliquer".into(),
            submit,
        });
    }

    /// Applique la règle dans un fil de travail et résume le résultat.
    fn rule_run(self: &Rc<Self>, id: &str) {
        let (root, jobs, dir, body, app) = (self.root.clone(), Arc::clone(&self.jobs), self.rules_dir(), json!({"id": id}), self.clone());
        spawn_work(
            move || run_rule(&root, &jobs, &dir, &body),
            move |result| {
                match result {
                    Ok(done) => app.toast(&format!("{} action(s) traitée(s)", done["count"])),
                    Err(refus) => app.toast(&refus.message),
                }
                app.reload();
                app.render_settings();
            },
        );
    }

    /// Annule la dernière exécution annulable et le dit.
    pub fn rules_undo(self: &Rc<Self>) {
        let (root, dir, app) = (self.root.clone(), self.rules_dir(), self.clone());
        spawn_work(
            move || undo_rules(&root, &dir),
            move |result| {
                match result {
                    Ok(done) => app.toast(&format!("{} action(s) annulée(s), {} impossible(s)", done["undone"], done["failed"].as_array().map_or(0, Vec::len))),
                    Err(refus) => app.toast(&refus.message),
                }
                app.reload();
                app.render_settings();
            },
        );
    }
}
