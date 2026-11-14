mod support;

use serde_json::{Value, json};
use support::rules_helpers::{codes, good};
use support::rules_json::{rule_from, rule_with};
#[test]
fn values_are_checked_regex_glob_length_and_templates() {
    let bad = |matcher: Value| rule_with(json!([{"test": {"type": "name", "matcher": matcher}}]), json!([{"type": "trash"}]));
    assert_eq!(codes(&bad(json!({"op": "regex", "value": "(oups"}))), ["matcher"]);
    assert_eq!(codes(&bad(json!({"op": "glob", "value": "*a*b*c*d*e*f*g*h*i*"}))), ["matcher"]);
    assert_eq!(codes(&bad(json!({"op": "contains", "value": "x".repeat(1001)}))), ["matcher"]);
}

#[test]
fn templates_are_checked_for_unknown_variables_and_braces() {
    let template = |t: &str| codes(&good(json!([{"type": "rename", "template": t}])));
    assert_eq!(template("{nom}"), ["template"]);
    assert_eq!(template("{stem"), ["template"]);
    assert!(template("{stem}-{modified:AAAA}").is_empty());
}

#[test]
fn contradictory_actions_and_plafonds_are_refused() {
    let mut rule = good(json!([{"type": "move_to", "folder": "Docs"}, {"type": "move_to", "folder": "Docs/Photos"}, {"type": "trash"}]));
    rule.watch[0].recursive = false;
    assert_eq!(codes(&rule), ["contradiction", "contradiction"]);
    let zipped = good(json!([{"type": "zip", "mode": "one_per_item"}, {"type": "unzip", "then": "keep"}]));
    assert_eq!(codes(&zipped), ["contradiction"]);
    let many = good(Value::Array((0..21).map(|_| json!({"type": "add_to_favorites"})).collect()));
    assert_eq!(codes(&many), ["limit"]);
}

#[test]
fn intervals_times_and_loops_are_checked() {
    let mut rule = good(json!([{"type": "add_to_favorites"}]));
    rule.triggers = serde_json::from_value(json!([{"type": "schedule", "every": {"n": 30, "unit": "seconds"}}, {"type": "stable", "for": {"n": 1, "unit": "seconds"}}, {"type": "daily", "at": "25:61"}])).unwrap();
    assert_eq!(codes(&rule), ["schedule", "stable", "time"]);
    let looping = rule_from(
        json!({"id": "l", "name": "l", "watch": [{"path": "Docs", "recursive": true}], "triggers": [{"type": "created"}], "alternatives": [{"conditions": [{"test": {"type": "is_file"}}]}], "actions": [{"type": "move_to", "folder": "Docs/Photos"}]}),
    );
    assert_eq!(codes(&looping), ["loop"]);
    let excluded = rule_from(
        json!({"id": "l", "name": "l", "watch": [{"path": "Docs", "recursive": true}], "triggers": [{"type": "created"}], "alternatives": [{"conditions": [{"test": {"type": "is_file"}}, {"negated": true, "test": {"type": "in_folder", "path": "Docs/Photos", "recursive": true}}]}], "actions": [{"type": "move_to", "folder": "Docs/Photos"}]}),
    );
    assert!(codes(&excluded).is_empty());
}
