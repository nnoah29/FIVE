mod support;

use five_engine::rules::eval::{condition_holds, rule_matches};
use serde_json::json;
use support::rules_json::{condition, file, rule_with};

fn holds(path: &str, test: serde_json::Value) -> bool {
    condition_holds(&file(path), &condition(json!({"test": test})))
}

#[test]
fn path_conditions_use_folder_depth_and_parent() {
    assert!(holds("Dl/a/b/c.txt", json!({"type": "in_folder", "path": "Dl", "recursive": true})));
    assert!(!holds("Dl/a/b/c.txt", json!({"type": "in_folder", "path": "Dl"})) && holds("Dl/c.txt", json!({"type": "in_folder", "path": "Dl"})));
    assert!(holds("Dl/a/b/c.txt", json!({"type": "depth", "cmp": "eq", "n": 3})));
    assert!(holds("Dl/Photos/c.txt", json!({"type": "parent_name", "matcher": {"op": "equals", "value": "photos"}})));
}

#[test]
fn negation_alternatives_and_groups_combine_with_or_and_and() {
    let rule = rule_with(
        json!([{"test": {"type": "extension", "in_list": ["pdf"]}}, {"negated": true, "test": {"type": "name_has_spaces"}}]),
        json!([{"type": "trash"}]),
    );
    assert!(rule_matches(&rule, &file("Dl/a.pdf")) && !rule_matches(&rule, &file("Dl/a b.pdf")) && !rule_matches(&rule, &file("Dl/a.txt")));
}
