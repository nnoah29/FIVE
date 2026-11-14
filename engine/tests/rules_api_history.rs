mod support;

use std::sync::Arc;

use five_engine::jobs::registry::Jobs;
use five_engine::rules::api_read::{rules_catalog, rules_history};
use five_engine::rules::api_run::{preview_rule, run_rule};
use five_engine::rules::api_write::{create_rule, enable_rule};
use serde_json::{Value, json};
use support::Sandbox;

fn body(action: Value) -> Value {
    json!({"name": "PDF", "watch": [{"path": "Dl"}], "triggers": [{"type": "manual"}],
        "alternatives": [{"conditions": [{"test": {"type": "extension", "in_list": ["pdf"]}}]}], "actions": [action]})
}

fn setup(action: Value) -> (Sandbox, std::path::PathBuf, String) {
    let sandbox = Sandbox::new();
    sandbox.write("Dl/a.pdf", "x");
    std::fs::create_dir_all(sandbox.dir.join("Docs")).unwrap();
    let dir = sandbox.dir.join(".config/five");
    let id = create_rule(&sandbox.root(), &dir, &body(action)).unwrap()["rule"]["id"].as_str().unwrap().to_string();
    (sandbox, dir, id)
}

#[test]
fn a_rule_in_trial_runs_as_a_dry_run_and_a_disabled_one_is_refused() {
    let (sandbox, dir, id) = setup(json!({"type": "trash"}));
    let jobs = Arc::new(Jobs::new());
    let trial = run_rule(&sandbox.root(), &jobs, &dir, &json!({"id": id})).unwrap();
    assert_eq!(trial["results"][0]["outcome"], "dry_run");
    assert!(sandbox.dir.join("Dl/a.pdf").exists());
    enable_rule(&sandbox.root(), &dir, &id, false, false).unwrap();
    assert_eq!(run_rule(&sandbox.root(), &jobs, &dir, &json!({"id": id})).unwrap_err().code, 409);
    assert_eq!(run_rule(&sandbox.root(), &jobs, &dir, &json!({"id": "r-x"})).unwrap_err().code, 404);
    assert_eq!(preview_rule(&sandbox.root(), &jobs, &dir, &json!({})).unwrap_err().code, 400);
}

#[test]
fn history_and_catalog_are_served_for_the_interface() {
    let (sandbox, dir, id) = setup(json!({"type": "trash"}));
    let jobs = Arc::new(Jobs::new());
    run_rule(&sandbox.root(), &jobs, &dir, &json!({"id": id})).unwrap();
    assert_eq!(rules_history(&dir, 10)["items"][0]["outcome"], "dry_run");
    assert_eq!(rules_history(&dir, 0)["items"].as_array().unwrap().len(), 1);
    assert_eq!(rules_catalog()["conditions"].as_array().unwrap().len(), 74);
}
