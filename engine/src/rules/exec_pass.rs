//! Exécution d'un plan complet : actions dans l'ordre par fichier, archives groupées, arrêt sur échec.

use std::collections::{BTreeMap, HashSet};

use super::exec::execute;
use super::exec_archive::make_zip;
use super::exec_common::action_name;
use super::outcome::PlannedAction;
use super::run_types::{ActionResult, Outcome, RunEnv, Undo};
use super::types_action::{RuleAction as A, ZipMode};

/// Résultat sans exécution (action écartée ou regroupée).
fn plain(planned: &PlannedAction, outcome: Outcome, detail: &str) -> ActionResult {
    ActionResult {
        rule_id: planned.rule_id.clone(),
        path: planned.path.clone(),
        action: action_name(&planned.action),
        outcome,
        detail: detail.to_string(),
        undo: Vec::new(),
    }
}

/// Compresse en une seule archive les fichiers regroupés par une règle `Zip { one_for_all }`.
fn zip_group(env: &RunEnv, items: &[&PlannedAction], current: &BTreeMap<String, String>) -> Vec<ActionResult> {
    let paths: Vec<String> = items.iter().filter_map(|p| current.get(&p.path).cloned()).collect();
    let outcome = make_zip(env, &paths, items[0].text.as_deref(), items[0].on_conflict);
    items
        .iter()
        .enumerate()
        .map(|(at, planned)| match &outcome {
            Ok(archive) => ActionResult {
                detail: env.root.relative(archive),
                undo: if at == 0 { vec![Undo::TrashCreated { path: archive.to_string_lossy().into_owned() }] } else { Vec::new() },
                ..plain(planned, Outcome::Done, "")
            },
            Err(fail) => plain(planned, Outcome::Failed, &format!("{fail:?}")),
        })
        .collect()
}

/// Exécute les actions prévues dans l'ordre ; après un échec ou une question, les actions suivantes du même fichier sont écartées.
pub fn run_plan(env: &RunEnv, planned: &[PlannedAction]) -> Vec<ActionResult> {
    let (mut results, mut current, mut halted) = (Vec::new(), BTreeMap::<String, String>::new(), HashSet::<String>::new());
    let mut groups: BTreeMap<String, Vec<&PlannedAction>> = BTreeMap::new();
    for item in planned {
        let here = current.entry(item.path.clone()).or_insert_with(|| item.path.clone());
        if matches!(item.action, A::Zip { mode: ZipMode::OneForAll, .. }) && !item.dry_run {
            groups.entry(item.rule_id.clone()).or_default().push(item);
        } else if halted.contains(&item.path) {
            results.push(plain(item, Outcome::Skipped, "action précédente non terminée"));
        } else {
            let result = execute(env, item, here);
            if matches!(result.outcome, Outcome::Failed | Outcome::Ask) {
                halted.insert(item.path.clone());
            }
            results.push(result);
        }
    }
    groups.values().for_each(|items| results.extend(zip_group(env, items, &current)));
    results
}
