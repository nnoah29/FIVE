mod support;

use std::fs;

use five_engine::rules::store::{Stored, load, rules_path, save};
use serde_json::json;
use support::Sandbox;
use support::rules_json::rule_with;

#[test]
fn rules_survive_a_round_trip_and_get_the_current_schema_version() {
    let sandbox = Sandbox::new();
    let path = rules_path(&sandbox.dir);
    let stored = Stored { rules: vec![rule_with(json!([{"test": {"type": "is_file"}}]), json!([{"type": "trash"}]))], ..Stored::default() };
    save(&path, &stored).unwrap();
    let back = load(&path);
    assert_eq!(back.rules.len(), 1);
    assert_eq!((back.rules[0].schema_version, back.damaged, back.newer), (1, false, false));
    assert_eq!(back.rules[0].actions, stored.rules[0].actions);
    assert!(!path.with_extension("json.tmp").exists());
}

#[test]
fn a_missing_file_is_an_empty_list() {
    let sandbox = Sandbox::new();
    let back = load(&rules_path(&sandbox.dir));
    assert!(back.rules.is_empty() && !back.damaged);
}

#[test]
fn a_damaged_file_is_set_aside_and_never_lost() {
    let sandbox = Sandbox::new();
    let path = rules_path(&sandbox.dir);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, "{ pas du json").unwrap();
    let back = load(&path);
    assert!(back.damaged && back.rules.is_empty());
    assert_eq!(fs::read_to_string(path.with_extension("json.abime")).unwrap(), "{ pas du json");
}

#[test]
fn rules_we_cannot_read_are_kept_untouched_on_save() {
    let sandbox = Sandbox::new();
    let path = rules_path(&sandbox.dir);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let future = json!({"id": "x", "name": "Future", "actions": [{"type": "action_du_futur"}]});
    fs::write(&path, json!({"schema_version": 1, "rules": [future]}).to_string()).unwrap();
    let mut stored = load(&path);
    assert_eq!((stored.rules.len(), stored.unreadable.len()), (0, 1));
    stored.rules.push(rule_with(json!([{"test": {"type": "is_file"}}]), json!([{"type": "trash"}])));
    save(&path, &stored).unwrap();
    let again = load(&path);
    assert_eq!((again.rules.len(), again.unreadable[0].clone()), (1, future));
}

#[test]
fn a_file_from_a_newer_version_is_preserved_whole() {
    let sandbox = Sandbox::new();
    let path = rules_path(&sandbox.dir);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, json!({"schema_version": 99, "rules": [{"id": "a"}]}).to_string()).unwrap();
    let stored = load(&path);
    assert!(stored.newer && stored.rules.is_empty() && stored.unreadable.len() == 1);
    save(&path, &stored).unwrap();
    assert_eq!(load(&path).unreadable.len(), 1);
}
