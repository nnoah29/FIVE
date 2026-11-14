mod support;

use five_engine::rules::context::FileContext;
use five_engine::rules::evaluate::{EventKind, evaluate};
use serde_json::json;
use support::rules_helpers::pdf_rule;
use support::rules_json::{file, rule_from};
#[test]
fn the_stable_trigger_waits_until_the_size_stops_changing() {
    let mut rule = pdf_rule("r1", json!([{"type": "trash"}]));
    rule.triggers = serde_json::from_value(json!([{"type": "stable", "for": {"n": 10, "unit": "seconds"}}])).unwrap();
    let at =
        |seconds: Option<f64>| evaluate(std::slice::from_ref(&rule), &FileContext { stable_for: seconds, ..file("Dl/a.pdf") }, EventKind::Stable).actions.len();
    assert_eq!((at(None), at(Some(3.0)), at(Some(10.0))), (0, 0, 1));
}

#[test]
fn trial_rules_and_log_only_plan_actions_without_executing_them() {
    let mut rule = pdf_rule("r1", json!([{"type": "trash"}]));
    rule.trial = true;
    assert!(evaluate(std::slice::from_ref(&rule), &file("Dl/a.pdf"), EventKind::Created).actions[0].dry_run);
    rule.trial = false;
    assert!(!evaluate(std::slice::from_ref(&rule), &file("Dl/a.pdf"), EventKind::Created).actions[0].dry_run);
    let logged = pdf_rule("r2", json!([{"type": "log_only"}, {"type": "trash"}]));
    assert!(evaluate(&[logged], &file("Dl/a.pdf"), EventKind::Created).actions[0].dry_run);
}

#[test]
fn moving_or_stopping_ends_the_pass_and_a_first_rename_wins() {
    let mover = pdf_rule("a", json!([{"type": "move_to", "folder": "Docs"}]));
    let after = pdf_rule("b", json!([{"type": "notify", "message_template": "x"}]));
    let out = evaluate(&[mover, after.clone()], &file("Dl/a.pdf"), EventKind::Created);
    assert_eq!((out.matched, out.actions.len()), (vec!["a".to_string()], 1));
    let stopper = rule_from(
        json!({"id": "s", "name": "s", "trial": false, "watch": [{"path": "Dl", "recursive": true}], "triggers": [{"type": "created"}], "alternatives": [{"conditions": [{"test": {"type": "is_file"}}]}], "actions": [{"type": "stop_processing"}]}),
    );
    assert_eq!(evaluate(&[stopper, after], &file("Dl/a.pdf"), EventKind::Created).matched, vec!["s".to_string()]);
    let (r1, r2) = (pdf_rule("1", json!([{"type": "slugify"}])), pdf_rule("2", json!([{"type": "add_prefix", "text": "x-"}])));
    let out = evaluate(&[r1, r2], &file("Dl/Été 1.pdf"), EventKind::Created);
    assert_eq!((out.actions.len(), out.skipped.len()), (1, 1));
}
