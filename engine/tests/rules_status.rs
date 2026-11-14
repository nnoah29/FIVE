mod support;

use five_engine::rules::status::{RuleStatus, rule_status};
use five_engine::rules::validate::{RootEnv, validate_set};
use serde_json::json;
use support::Sandbox;
use support::rules_json::{rule_from, rule_with};

#[test]
fn the_status_is_invalid_then_disabled_then_trial_then_active() {
    let sandbox = Sandbox::new();
    std::fs::create_dir_all(sandbox.dir.join("Dl")).unwrap();
    let root = sandbox.root();
    let env = RootEnv(&root);
    let mut rule = rule_with(json!([{"test": {"type": "is_file"}}]), json!([{"type": "add_to_favorites"}]));
    rule.watch[0].recursive = false;
    assert_eq!(rule_status(&rule, &env), RuleStatus::Active);
    rule.trial = true;
    assert_eq!(rule_status(&rule, &env), RuleStatus::Trial);
    rule.enabled = false;
    assert_eq!(rule_status(&rule, &env).as_str(), "disabled");
    rule.watch[0].path = "Disparu".into();
    assert!(matches!(rule_status(&rule, &env), RuleStatus::Invalid(reason) if reason.contains("/Disparu")));
}

#[test]
fn the_real_root_environment_detects_missing_outside_and_trash_folders() {
    let sandbox = Sandbox::new();
    std::fs::create_dir_all(sandbox.dir.join(".local/share/Trash/files")).unwrap();
    sandbox.write("fichier.txt", "x");
    let root = sandbox.root();
    let env = RootEnv(&root);
    let at = |path: &str| {
        let mut rule = rule_with(json!([{"test": {"type": "is_file"}}]), json!([{"type": "add_to_favorites"}]));
        rule.watch = serde_json::from_value(json!([{"path": path}])).unwrap();
        rule_status(&rule, &env)
    };
    assert!(matches!(at("absent"), RuleStatus::Invalid(r) if r.contains("n'existe pas")));
    assert!(matches!(at("fichier.txt"), RuleStatus::Invalid(r) if r.contains("n'existe pas")));
    assert!(matches!(at("../.."), RuleStatus::Invalid(r) if r.contains("hors de la racine")));
    assert!(matches!(at(".local/share/Trash/files"), RuleStatus::Invalid(r) if r.contains("corbeille")));
}

#[test]
fn the_set_needs_unique_identifiers_and_at_most_200_rules() {
    let make = |id: &str| rule_from(json!({"id": id, "name": "n", "watch": [], "triggers": [], "alternatives": [], "actions": []}));
    assert!(validate_set(&[make("a"), make("b")]).is_empty());
    assert_eq!(validate_set(&[make("a"), make("a"), make(" ")]).len(), 2);
    let many: Vec<_> = (0..201).map(|n| make(&format!("r{n}"))).collect();
    assert_eq!(validate_set(&many).len(), 1);
}
