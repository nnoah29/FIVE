//! Actions manuelles du service des règles : appliquer maintenant, aperçu.

use super::evaluate::EventKind;
use super::ids_run::new_run_id;
use super::journal::{append, journal_path};
use super::outcome::PlannedAction;
use super::runner::{RulesService, TickReport};
use super::runner_eval::{Item, Pass, plan_files};
use super::runner_events::full_scan;
use super::runner_info::mount_names;
use super::runner_rules::active_rules;
use super::types_rule::Rule;

impl RulesService {
    /// Fichiers à traiter pour « Appliquer maintenant » : ceux qu'on donne, sinon tout le dossier surveillé.
    fn manual_items(&self, rules: &[Rule], paths: Option<&[String]>) -> Vec<Item> {
        match paths {
            Some(list) => list.iter().map(|p| Item::new(p, EventKind::Manual, None)).collect(),
            None => full_scan(&self.root, rules, EventKind::Manual),
        }
    }

    /// « Appliquer maintenant » : une règle (ou toutes) sur des fichiers précis (ou tout leur dossier surveillé).
    pub fn run_now(&mut self, rule_id: Option<&str>, paths: Option<&[String]>, now: f64) -> TickReport {
        let mut rules = active_rules(&self.root, &self.dir);
        rules.retain(|r| rule_id.is_none_or(|id| r.id == id));
        let items = self.manual_items(&rules, paths);
        let results = self.execute(&rules, &items, now);
        let report = TickReport { run: new_run_id(), changes: items.len(), results, incomplete: false };
        let _ = append(&journal_path(&self.dir), &report.run, &report.results);
        report
    }

    /// Ce qu'une règle ferait, sans rien modifier (toutes les actions sont marquées « essai »).
    pub fn preview(&self, rule: &Rule, paths: Option<&[String]>, now: f64) -> Vec<PlannedAction> {
        let mut trial = rule.clone();
        (trial.enabled, trial.trial) = (true, true);
        let rules = [trial];
        let items = self.manual_items(&rules, paths);
        let disks = mount_names(&self.root);
        plan_files(&Pass { root: &self.root, rules: &rules, extras: &self.extras, disks: &disks, now }, &items).0
    }
}
