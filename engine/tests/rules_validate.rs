mod support;

use serde_json::json;
use support::rules_helpers::{codes, good};
#[test]
fn a_complete_rule_is_valid() {
    assert!(codes(&good(json!([{"type": "move_to", "folder": "Docs"}]))).is_empty());
    assert!(codes(&good(json!([{"type": "move_to_dated", "folder": "Docs", "pattern": "{modified:AAAA}"}]))).is_empty());
}

#[test]
fn missing_parts_are_reported_one_by_one() {
    let mut rule = good(json!([]));
    rule.name = " ".into();
    rule.watch.clear();
    rule.triggers.clear();
    rule.alternatives = serde_json::from_value(json!([{"conditions": []}])).unwrap();
    assert_eq!(codes(&rule), ["name", "watch", "trigger", "conditions", "actions"]);
}

#[test]
fn folders_must_exist_stay_in_the_root_and_avoid_the_trash() {
    let mut rule = good(json!([{"type": "move_to", "folder": "Absent"}]));
    rule.watch = serde_json::from_value(json!([{"path": "../hors"}, {"path": "Corbeille"}, {"path": "Dl"}])).unwrap();
    assert_eq!(codes(&rule), ["folder", "folder", "folder"]);
    let mut cleanup = good(json!([{"type": "empty_trash_older_than", "duration": {"n": 30, "unit": "days"}}]));
    cleanup.watch = serde_json::from_value(json!([{"path": "Corbeille"}])).unwrap();
    assert!(codes(&cleanup).is_empty());
}
