//! Évaluation d'un lot de fichiers : contexte, règles, plan d'actions, plafond de fichiers par règle.

use std::collections::{HashMap, HashSet};

use super::context_build::{BuildEnv, Extras, build_context};
use super::evaluate::{EventKind, evaluate};
use super::needs::needs_of;
use super::outcome::PlannedAction;
use super::run_types::{ActionResult, Outcome};
use super::types_rule::{Rule, Watch};
use crate::root::Root;

const MAX_FILES_PER_RULE: usize = 100;

/// Une passe : ce qu'il faut pour évaluer des fichiers.
pub struct Pass<'a> {
    pub root: &'a Root,
    pub rules: &'a [Rule],
    pub extras: &'a Extras,
    pub disks: &'a [String],
    pub now: f64,
}

/// Fichier à évaluer : chemin, événement, durée de stabilité, et la seule règle concernée (événements planifiés).
#[derive(Debug, Clone)]
pub struct Item {
    pub path: String,
    pub event: EventKind,
    pub stable_for: Option<f64>,
    pub only_rule: Option<String>,
}

impl Item {
    /// Fichier évalué par toutes les règles qui surveillent son dossier.
    pub fn new(path: &str, event: EventKind, stable_for: Option<f64>) -> Item {
        Item { path: path.to_string(), event, stable_for, only_rule: None }
    }
}

/// Dossiers surveillés de toutes les règles.
pub fn watches_of(rules: &[Rule]) -> Vec<Watch> {
    rules.iter().flat_map(|r| r.watch.clone()).collect()
}

/// Évalue chaque fichier avec les règles ; au-delà de 100 fichiers par règle, le reste est écarté avec une raison.
pub fn plan_files(pass: &Pass, items: &[Item]) -> (Vec<PlannedAction>, Vec<ActionResult>) {
    let refs: Vec<&Rule> = pass.rules.iter().collect();
    let watches = watches_of(pass.rules);
    let env = BuildEnv { root: pass.root, needs: needs_of(&refs), extras: pass.extras, watches: &watches, disks: pass.disks, now: pass.now };
    let (mut planned, mut limited) = (Vec::new(), Vec::new());
    let mut files: HashMap<String, HashSet<String>> = HashMap::new();
    for (index, item) in items.iter().enumerate() {
        let (path, Some(ctx)) = (&item.path, build_context(&env, &item.path, item.stable_for, index as u32)) else { continue };
        let targeted: Vec<Rule> = pass.rules.iter().filter(|r| item.only_rule.as_ref().is_none_or(|id| &r.id == id)).cloned().collect();
        for action in evaluate(&targeted, &ctx, item.event).actions {
            let seen = files.entry(action.rule_id.clone()).or_default();
            seen.insert(path.clone());
            if seen.len() > MAX_FILES_PER_RULE {
                limited.push(ActionResult {
                    rule_id: action.rule_id.clone(),
                    path: path.clone(),
                    action: "limit".into(),
                    outcome: Outcome::Skipped,
                    detail: "plafond de 100 fichiers par passe atteint".into(),
                    undo: Vec::new(),
                });
            } else {
                planned.push(action);
            }
        }
    }
    (planned, limited)
}
