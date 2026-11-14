mod support;

use five_engine::rules::api_read::list_rules;
use five_engine::rules::api_write::{create_rule, delete_rule, reorder_rules};
use serde_json::{Value, json};
use support::Sandbox;

fn body(name: &str, watch: &str) -> Value {
    json!({"name": name, "watch": [{"path": watch}], "triggers": [{"type": "created"}], "trial": false,
        "alternatives": [{"conditions": [{"test": {"type": "extension", "in_list": ["pdf"]}}]}], "actions": [{"type": "add_to_favorites"}]})
}

fn setup() -> (Sandbox, std::path::PathBuf) {
    let sandbox = Sandbox::new();
    std::fs::create_dir_all(sandbox.dir.join("Dl")).unwrap();
    let dir = sandbox.dir.join(".config/five");
    (sandbox, dir)
}

#[test]
fn rules_can_be_deleted_and_reordered_only_with_the_exact_set() {
    let (sandbox, dir) = setup();
    let ids: Vec<String> =
        ["A", "B", "C"].iter().map(|n| create_rule(&sandbox.root(), &dir, &body(n, "Dl")).unwrap()["rule"]["id"].as_str().unwrap().to_string()).collect();
    reorder_rules(&dir, &[ids[2].clone(), ids[0].clone(), ids[1].clone()]).unwrap();
    let names = |v: Value| v["rules"].as_array().unwrap().iter().map(|r| r["name"].as_str().unwrap().to_string()).collect::<Vec<_>>();
    assert_eq!(names(list_rules(&sandbox.root(), &dir)), ["C", "A", "B"]);
    assert_eq!(reorder_rules(&dir, &ids[..2]).unwrap_err().code, 400);
    delete_rule(&dir, &ids[0]).unwrap();
    assert_eq!(delete_rule(&dir, &ids[0]).unwrap_err().code, 404);
    assert_eq!(names(list_rules(&sandbox.root(), &dir)), ["C", "B"]);
}

#[test]
fn the_list_shows_why_a_rule_became_invalid_and_flags_damaged_files() {
    let (sandbox, dir) = setup();
    create_rule(&sandbox.root(), &dir, &body("A", "Dl")).unwrap();
    std::fs::remove_dir(sandbox.dir.join("Dl")).unwrap();
    let rule = &list_rules(&sandbox.root(), &dir)["rules"][0];
    assert_eq!(rule["status"], "invalid");
    assert!(rule["reason"].as_str().unwrap().contains("/Dl"));
    std::fs::write(dir.join("regles.json"), "{ cassé").unwrap();
    assert_eq!(list_rules(&sandbox.root(), &dir)["damaged"], true);
}
