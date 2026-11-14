mod support;

use std::fs;
use std::sync::Arc;

use five_engine::jobs::registry::Jobs;
use five_engine::rules::runner::RulesService;
use five_engine::rules::state::{load_state, state_path};
use five_engine::rules::store::{Stored, save};
use five_engine::rules::types_rule::Rule;
use serde_json::{Value, json};
use support::Sandbox;

fn rule(trigger: Value, watch: &str) -> Rule {
    serde_json::from_value(json!({
        "id": "r1", "name": "n", "trial": false, "watch": [{"path": watch}], "triggers": [trigger],
        "alternatives": [{"conditions": [{"test": {"type": "is_file"}}]}], "actions": [{"type": "add_to_favorites"}]
    }))
    .unwrap()
}

fn service(sandbox: &Sandbox, rules: Vec<Rule>) -> RulesService {
    fs::create_dir_all(sandbox.dir.join("Dl")).unwrap();
    let dir = sandbox.dir.join(".config/five");
    save(&dir.join("regles.json"), &Stored { rules, ..Stored::default() }).unwrap();
    RulesService::new(sandbox.root(), Arc::new(Jobs::new()), dir)
}

#[test]
fn an_on_start_rule_runs_over_existing_files_on_the_first_pass_only() {
    let sandbox = Sandbox::new();
    sandbox.write("Dl/a.txt", "x");
    let mut svc = service(&sandbox, vec![rule(json!({"type": "on_start"}), "Dl")]);
    assert_eq!(svc.tick(100.0).results.len(), 1);
    assert!(svc.tick(200.0).results.is_empty());
}

#[test]
fn a_scheduled_rule_is_remembered_first_then_runs_when_due() {
    let sandbox = Sandbox::new();
    sandbox.write("Dl/a.txt", "x");
    let mut svc = service(&sandbox, vec![rule(json!({"type": "schedule", "every": {"n": 10, "unit": "minutes"}}), "Dl")]);
    assert!(svc.tick(1000.0).results.is_empty());
    assert!(svc.tick(1300.0).results.is_empty());
    assert_eq!(svc.tick(1700.0).results.len(), 1);
    assert_eq!(load_state(&state_path(&sandbox.dir.join(".config/five"))).last_run["r1"], 1700.0);
}

#[test]
fn an_invalid_rule_is_never_applied() {
    let sandbox = Sandbox::new();
    sandbox.write("Dl/a.txt", "x");
    let mut svc = service(&sandbox, vec![rule(json!({"type": "on_start"}), "Disparu")]);
    let report = svc.tick(100.0);
    assert!(report.results.is_empty() && report.changes == 0);
}
