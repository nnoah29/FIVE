//! Événements autres que les changements de fichiers : règles planifiées, démarrage, montage d'un disque.

use std::time::{Duration, Instant};

use super::evaluate::EventKind;
use super::runner_eval::{Item, watches_of};
use super::scan::scan_watch;
use super::schedule::due_events;
use super::state::RulesState;
use super::types_rule::{Rule, Trigger};
use crate::root::Root;

const FULL_SCAN: Duration = Duration::from_secs(5);

/// Tous les fichiers surveillés par ces règles, avec l'événement donné.
pub fn full_scan(root: &Root, rules: &[Rule], event: EventKind) -> Vec<Item> {
    let mut seen = Vec::new();
    let deadline = Instant::now() + FULL_SCAN;
    for watch in watches_of(rules) {
        scan_watch(root, &watch, deadline, &mut seen);
    }
    seen.sort_by(|a, b| a.path.cmp(&b.path));
    seen.dedup_by(|a, b| a.path == b.path);
    seen.into_iter().map(|s| Item::new(&s.path, event, None)).collect()
}

/// Fichiers à évaluer pour une règle ciblée, avec son événement.
fn targeted(root: &Root, rule: &Rule, event: EventKind) -> Vec<Item> {
    full_scan(root, std::slice::from_ref(rule), event).into_iter().map(|item| Item { only_rule: Some(rule.id.clone()), ..item }).collect()
}

/// Règles planifiées à échéance ; une règle vue pour la première fois est seulement mémorisée.
fn scheduled_items(root: &Root, rules: &[Rule], state: &mut RulesState, now: f64) -> Vec<Item> {
    let mut items = Vec::new();
    for (id, event) in due_events(rules, state, now) {
        state.last_run.insert(id.clone(), now);
        items.extend(rules.iter().filter(|r| r.id == id).flat_map(|r| targeted(root, r, event)));
    }
    for rule in rules.iter().filter(|r| r.triggers.iter().any(|t| matches!(t, Trigger::Schedule { .. } | Trigger::Daily { .. } | Trigger::Weekly { .. }))) {
        state.last_run.entry(rule.id.clone()).or_insert(now);
    }
    items
}

/// Règles à lancer pour un événement fixe : démarrage, ou disque nouvellement monté.
fn fixed_items(root: &Root, rules: &[Rule], event: EventKind, wanted: impl Fn(&Trigger) -> bool) -> Vec<Item> {
    rules.iter().filter(|r| r.triggers.iter().any(&wanted)).flat_map(|r| targeted(root, r, event)).collect()
}

/// Tous les éléments à évaluer pour les événements non liés à un changement de fichier.
pub fn planned_items(root: &Root, rules: &[Rule], state: &mut RulesState, now: f64, first_tick: bool, disks: &[String]) -> Vec<Item> {
    let mut items = scheduled_items(root, rules, state, now);
    if first_tick {
        items.extend(fixed_items(root, rules, EventKind::OnStart, |t| matches!(t, Trigger::OnStart)));
    }
    let new_disks: Vec<&String> = if state.disks.is_empty() { Vec::new() } else { disks.iter().filter(|d| !state.disks.contains(d)).collect() };
    for disk in new_disks {
        items.extend(fixed_items(root, rules, EventKind::OnMount, |t| matches!(t, Trigger::OnMount { disk: wanted } if wanted.eq_ignore_ascii_case(disk))));
    }
    state.disks = disks.to_vec();
    items
}
