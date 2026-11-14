//! Service des règles : un appel à `tick` balaie les dossiers, détecte les changements, évalue, exécute, journalise.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use super::changes::{Change, ChangeKind, detect};
use super::context_build::Extras;
use super::evaluate::EventKind;
use super::exec_pass::run_plan;
use super::ids_run::new_run_id;
use super::journal::{append, journal_path};
use super::run_types::{ActionResult, RunEnv};
use super::runner_eval::{Item, Pass, plan_files, watches_of};
use super::runner_events::planned_items;
use super::runner_info::{max_wait, mount_names};
use super::runner_limits::limit_notifications;
use super::runner_rules::active_rules;
use super::scan::{Seen, scan_watch};
use super::state::{load_state, save_state, state_path};
use super::types_rule::Rule;
use crate::jobs::registry::Jobs;
use crate::root::Root;

const SCAN_BUDGET: Duration = Duration::from_millis(200);

/// Bilan d'une passe.
#[derive(Debug, Clone, Default)]
pub struct TickReport {
    pub run: String,
    pub changes: usize,
    pub results: Vec<ActionResult>,
    pub incomplete: bool,
}

/// Veilleur et exécuteur des règles.
pub struct RulesService {
    pub root: Root,
    pub jobs: Arc<Jobs>,
    pub dir: PathBuf,
    pub extras: Extras,
    started: bool,
}

/// Événement correspondant à un changement de fichier.
fn event_of(change: &Change) -> Item {
    let event = match change.kind {
        ChangeKind::Created => EventKind::Created,
        ChangeKind::Modified => EventKind::Modified,
        ChangeKind::Stable => EventKind::Stable,
    };
    Item::new(&change.path, event, Some(change.stable_for))
}

impl RulesService {
    /// Crée le service ; `dir` est le dossier de configuration (`~/.config/five`).
    pub fn new(root: Root, jobs: Arc<Jobs>, dir: PathBuf) -> RulesService {
        RulesService { root, jobs, dir, extras: Extras::default(), started: false }
    }

    /// Évalue les éléments puis exécute le plan.
    pub(super) fn execute(&self, rules: &[Rule], items: &[Item], now: f64) -> Vec<ActionResult> {
        let disks = mount_names(&self.root);
        let pass = Pass { root: &self.root, rules, extras: &self.extras, disks: &disks, now };
        let (planned, mut results) = plan_files(&pass, items);
        results.extend(run_plan(&RunEnv { root: &self.root, jobs: &self.jobs }, &planned));
        results
    }

    /// Balaie les dossiers surveillés (200 ms au plus) ; retourne ce qui a vu et si le balayage est complet.
    pub(super) fn scan(&self, rules: &[Rule]) -> (Vec<Seen>, bool) {
        let (mut seen, deadline) = (Vec::new(), Instant::now() + SCAN_BUDGET);
        let complete = watches_of(rules).iter().all(|w| scan_watch(&self.root, w, deadline, &mut seen));
        (seen, complete)
    }

    /// Une passe : changements des dossiers surveillés, règles planifiées, démarrage, montages ; exécute et journalise.
    pub fn tick(&mut self, now: f64) -> TickReport {
        let rules = active_rules(&self.root, &self.dir);
        let mut report = TickReport { run: new_run_id(), ..TickReport::default() };
        if rules.is_empty() {
            return report;
        }
        let mut state = load_state(&state_path(&self.dir));
        let (seen, complete) = self.scan(&rules);
        let changes = detect(&mut state, &seen, now, max_wait(&rules), complete);
        let mut items: Vec<Item> = changes.iter().map(event_of).collect();
        items.extend(planned_items(&self.root, &rules, &mut state, now, !self.started, &mount_names(&self.root)));
        self.started = true;
        (report.changes, report.incomplete) = (items.len(), !complete);
        report.results = limit_notifications(&mut state, self.execute(&rules, &items, now), now);
        let _ = save_state(&state_path(&self.dir), &state);
        let _ = append(&journal_path(&self.dir), &report.run, &report.results);
        report
    }
}
