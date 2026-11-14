mod support;

use std::sync::Arc;

use five_engine::jobs::registry::Jobs;
use five_engine::rules::api_run::{preview_rule, run_rule, undo_rules, validate_body};
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
fn validate_lists_structured_problems_without_saving() {
    let sandbox = Sandbox::new();
    let mut bad = body(json!({"type": "trash"}));
    bad["watch"] = json!([{"path": "Nulle part"}]);
    let answer = validate_body(&sandbox.root(), &bad).unwrap();
    assert_eq!((answer["valid"].as_bool(), answer["problems"][0]["code"].as_str()), (Some(false), Some("folder")));
    assert!(!sandbox.dir.join(".config").exists());
}

#[test]
fn preview_lists_what_would_happen_and_is_required_before_activating() {
    let (sandbox, dir, id) = setup(json!({"type": "move_to", "folder": "Docs"}));
    let jobs = Arc::new(Jobs::new());
    let refus = enable_rule(&sandbox.root(), &dir, &id, true, true).unwrap_err();
    assert_eq!(refus.code, 409);
    let seen = preview_rule(&sandbox.root(), &jobs, &dir, &json!({"id": id})).unwrap();
    assert_eq!((seen["count"].as_u64(), seen["items"][0]["text"].as_str(), seen["items"][0]["dry_run"].as_bool()), (Some(1), Some("Docs"), Some(true)));
    assert!(sandbox.dir.join("Dl/a.pdf").exists());
    let active = enable_rule(&sandbox.root(), &dir, &id, true, true).unwrap();
    assert_eq!(active["rule"]["status"], "active");
}

#[test]
fn run_applies_the_rule_and_undo_restores_it() {
    let (sandbox, dir, id) = setup(json!({"type": "move_to", "folder": "Docs"}));
    let jobs = Arc::new(Jobs::new());
    preview_rule(&sandbox.root(), &jobs, &dir, &json!({"id": id})).unwrap();
    enable_rule(&sandbox.root(), &dir, &id, true, true).unwrap();
    let done = run_rule(&sandbox.root(), &jobs, &dir, &json!({"id": id})).unwrap();
    assert_eq!((done["count"].as_u64(), done["results"][0]["outcome"].as_str()), (Some(1), Some("done")));
    assert!(sandbox.dir.join("Docs/a.pdf").exists());
    let undone = undo_rules(&sandbox.root(), &dir).unwrap();
    assert_eq!(undone["undone"], 1);
    assert!(sandbox.dir.join("Dl/a.pdf").exists());
    assert_eq!(undo_rules(&sandbox.root(), &dir).unwrap_err().code, 404);
}
