mod support;

use five_engine::rules::context::FileContext;
use five_engine::rules::evaluate::{EventKind, evaluate, watch_contains};
use serde_json::json;
use support::rules_helpers::pdf_rule;
use support::rules_json::{file, rule_with};
#[test]
fn a_rule_only_acts_inside_its_watched_folders_for_its_triggers() {
    let rule = pdf_rule("r1", json!([{"type": "trash"}]));
    let acts = |ctx: &FileContext, event| !evaluate(std::slice::from_ref(&rule), ctx, event).actions.is_empty();
    assert!(acts(&file("Dl/a.pdf"), EventKind::Created) && acts(&file("Dl/x/y/a.pdf"), EventKind::Created));
    assert!(
        !acts(&file("Docs/a.pdf"), EventKind::Created) && !acts(&file("Dl/a.pdf"), EventKind::Modified) && !acts(&file("Dl/.cache/a.pdf"), EventKind::Created)
    );
    assert!(!acts(&file("Docs/a.pdf"), EventKind::Manual) && acts(&file("Dl/a.pdf"), EventKind::Manual));
}

#[test]
fn watched_folders_honor_recursion_depth_and_hidden_items() {
    let watch = |value| serde_json::from_value(value).unwrap();
    let flat = watch(json!({"path": "Dl"}));
    let deep = watch(json!({"path": "Dl", "recursive": true, "max_depth": 1}));
    assert!(watch_contains(&flat, "Dl/a.txt") && !watch_contains(&flat, "Dl/x/a.txt") && !watch_contains(&flat, "Dl") && !watch_contains(&flat, "Dls/a.txt"));
    assert!(watch_contains(&deep, "Dl/x/a.txt") && !watch_contains(&deep, "Dl/x/y/a.txt"));
    assert!(
        watch_contains(&watch(json!({"path": "", "recursive": true})), "a/b.txt")
            && !watch_contains(&watch(json!({"path": "Dl", "ignore_hidden": true})), "Dl/.a")
    );
}

#[test]
fn disabled_rules_and_temporary_files_are_ignored_except_on_demand() {
    let mut rule = pdf_rule("r1", json!([{"type": "trash"}]));
    rule.enabled = false;
    assert!(evaluate(&[rule.clone()], &file("Dl/a.pdf"), EventKind::Manual).actions.is_empty());
    rule.enabled = true;
    let temp = file("Dl/b.pdf.part");
    let tmp_rule = rule_with(json!([{"test": {"type": "is_file"}}]), json!([{"type": "trash"}]));
    assert!(
        evaluate(std::slice::from_ref(&tmp_rule), &temp, EventKind::Created).actions.is_empty()
            && !evaluate(&[tmp_rule], &temp, EventKind::Manual).actions.is_empty()
    );
}
