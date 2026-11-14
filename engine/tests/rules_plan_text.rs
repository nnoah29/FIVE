mod support;

use five_engine::rules::evaluate::{EventKind, evaluate};
use serde_json::json;
use support::rules_helpers::{dated, texts};
use support::rules_json::{file, rule_with};
#[test]
fn rename_templates_fill_every_kind_of_variable() {
    let out = texts(&dated(), json!([{"type": "rename", "template": "{modified:AAAA-MM-JJ}_{stem}_{counter:3}.{ext}"}]));
    assert_eq!(out, vec![Some("2024-01-15_Mon Été_004.jpeg".to_string())]);
    let today = texts(&dated(), json!([{"type": "rename", "template": "{today:AAAA}-{kind}-{parent}"}]));
    assert_eq!(today, vec![Some("2023-image-Dl".to_string())]);
}

#[test]
fn an_unknown_or_unavailable_variable_skips_the_action_with_a_reason() {
    let rule = rule_with(json!([{"test": {"type": "is_file"}}]), json!([{"type": "rename", "template": "{width}x{height}"}]));
    let out = evaluate(&[rule], &file("Dl/a.png"), EventKind::Created);
    assert!(out.actions.is_empty() && out.skipped[0].reason.contains("width"));
}

#[test]
fn name_operations_chain_on_the_running_name() {
    let out = texts(&dated(), json!([{"type": "slugify"}, {"type": "add_prefix", "text": "x-"}, {"type": "normalize_extension"}]));
    assert_eq!(out, vec![Some("mon-ete.jpeg".to_string()), Some("x-mon-ete.jpeg".to_string()), Some("x-mon-ete.jpg".to_string())]);
}
