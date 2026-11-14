mod support;

use five_engine::rules::api_read::list_rules;
use five_engine::rules::api_write::{create_rule, update_rule};
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
fn a_created_rule_gets_an_id_and_always_starts_in_trial() {
    let (sandbox, dir) = setup();
    let made = create_rule(&sandbox.root(), &dir, &body("PDF", "Dl")).unwrap();
    assert!(made["rule"]["id"].as_str().unwrap().starts_with("r-") && made["rule"]["trial"] == true);
    assert_eq!(made["rule"]["status"], "trial");
    assert_eq!(list_rules(&sandbox.root(), &dir)["rules"].as_array().unwrap().len(), 1);
}

#[test]
fn an_invalid_rule_is_refused_with_every_problem_listed() {
    let (sandbox, dir) = setup();
    let mut bad = body("", "Absent");
    bad["actions"] = json!([{"type": "rename", "template": "{nom}"}]);
    let refus = create_rule(&sandbox.root(), &dir, &bad).unwrap_err();
    assert_eq!(refus.code, 400);
    assert!(["nom", "/Absent", "{nom}"].iter().all(|part| refus.message.contains(part)), "{}", refus.message);
    assert_eq!(create_rule(&sandbox.root(), &dir, &json!({"name": 3})).unwrap_err().code, 400);
    assert!(list_rules(&sandbox.root(), &dir)["rules"].as_array().unwrap().is_empty());
}

#[test]
fn updating_keeps_the_id_goes_back_to_trial_and_unknown_ids_are_404() {
    let (sandbox, dir) = setup();
    let id = create_rule(&sandbox.root(), &dir, &body("A", "Dl")).unwrap()["rule"]["id"].as_str().unwrap().to_string();
    let changed = update_rule(&sandbox.root(), &dir, &id, &body("B", "Dl")).unwrap();
    assert_eq!(
        (changed["rule"]["id"].as_str(), changed["rule"]["name"].as_str(), changed["rule"]["trial"].as_bool()),
        (Some(id.as_str()), Some("B"), Some(true))
    );
    assert_eq!(update_rule(&sandbox.root(), &dir, "r-inconnu", &body("C", "Dl")).unwrap_err().code, 404);
}
