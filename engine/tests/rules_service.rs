mod support;

use std::fs;
use std::sync::Arc;

use five_engine::jobs::registry::Jobs;
use five_engine::rules::journal::{journal_path, read_recent};
use five_engine::rules::run_types::Outcome;
use five_engine::rules::runner::RulesService;
use five_engine::rules::store::{Stored, save};
use five_engine::rules::types_rule::Rule;
use serde_json::json;
use support::Sandbox;

fn rule(trigger: serde_json::Value, actions: serde_json::Value, trial: bool) -> Rule {
    serde_json::from_value(json!({
        "id": "r1", "name": "PDF", "trial": trial,
        "watch": [{"path": "Dl"}], "triggers": [trigger],
        "alternatives": [{"conditions": [{"test": {"type": "extension", "in_list": ["pdf"]}}]}],
        "actions": actions
    }))
    .unwrap()
}

fn service(sandbox: &Sandbox, rules: Vec<Rule>) -> RulesService {
    for folder in ["Dl", "Docs"] {
        fs::create_dir_all(sandbox.dir.join(folder)).unwrap();
    }
    let dir = sandbox.dir.join(".config/five");
    save(&dir.join("regles.json"), &Stored { rules, ..Stored::default() }).unwrap();
    RulesService::new(sandbox.root(), Arc::new(Jobs::new()), dir)
}

#[test]
fn existing_files_are_left_alone_and_a_new_one_is_moved() {
    let sandbox = Sandbox::new();
    sandbox.write("Dl/vieux.pdf", "1");
    let mut svc = service(&sandbox, vec![rule(json!({"type": "created"}), json!([{"type": "move_to", "folder": "Docs"}]), false)]);
    assert!(svc.tick(100.0).results.is_empty());
    sandbox.write("Dl/nouveau.pdf", "2");
    let report = svc.tick(105.0);
    assert_eq!((report.results.len(), report.results[0].outcome), (1, Outcome::Done));
    assert!(sandbox.dir.join("Docs/nouveau.pdf").is_file() && sandbox.dir.join("Dl/vieux.pdf").is_file());
    assert!(svc.tick(110.0).results.is_empty());
}

#[test]
fn a_rule_in_trial_only_journals_what_it_would_do() {
    let sandbox = Sandbox::new();
    let mut svc = service(&sandbox, vec![rule(json!({"type": "created"}), json!([{"type": "trash"}]), true)]);
    svc.tick(100.0);
    sandbox.write("Dl/a.pdf", "x");
    let report = svc.tick(105.0);
    assert_eq!(report.results[0].outcome, Outcome::DryRun);
    assert!(sandbox.dir.join("Dl/a.pdf").exists());
    assert_eq!(read_recent(&journal_path(&sandbox.dir.join(".config/five")), 5)[0]["outcome"], "dry_run");
}

#[test]
fn the_stable_trigger_waits_for_the_file_to_stop_changing() {
    let sandbox = Sandbox::new();
    let stable = json!({"type": "stable", "for": {"n": 10, "unit": "seconds"}});
    let mut svc = service(&sandbox, vec![rule(stable, json!([{"type": "move_to", "folder": "Docs"}]), false)]);
    svc.tick(100.0);
    sandbox.write("Dl/a.pdf", "x");
    assert!(svc.tick(101.0).results.is_empty() && svc.tick(105.0).results.is_empty());
    assert_eq!(svc.tick(112.0).results[0].outcome, Outcome::Done);
    assert!(sandbox.dir.join("Docs/a.pdf").exists());
}
