use five_engine::rules::evaluate::EventKind;
use five_engine::rules::schedule::due_events;
use five_engine::rules::state::RulesState;
use five_engine::rules::types_rule::Rule;
use serde_json::{Value, json};

fn rule(trigger: Value) -> Rule {
    serde_json::from_value(json!({"id": "r", "name": "r", "watch": [{"path": "Dl"}], "triggers": [trigger], "alternatives": [{"conditions": [{"test": {"type": "is_file"}}]}], "actions": [{"type": "trash"}]})).unwrap()
}

fn due(trigger: Value, last: Option<f64>, now: f64) -> Vec<(String, EventKind)> {
    let mut state = RulesState::default();
    if let Some(at) = last {
        state.last_run.insert("r".into(), at);
    }
    due_events(&[rule(trigger)], &state, now)
}

#[test]
fn an_interval_is_due_once_enough_time_has_passed() {
    let every = json!({"type": "schedule", "every": {"n": 10, "unit": "minutes"}});
    assert!(due(every.clone(), Some(1000.0), 1500.0).is_empty());
    assert_eq!(due(every, Some(1000.0), 1600.0), [("r".to_string(), EventKind::Schedule)]);
}

#[test]
fn a_rule_never_seen_before_is_not_due_yet() {
    assert!(due(json!({"type": "schedule", "every": {"n": 1, "unit": "minutes"}}), None, 99_999.0).is_empty());
}

#[test]
fn disabled_rules_and_other_triggers_never_come_due() {
    let mut state = RulesState::default();
    state.last_run.insert("r".into(), 0.0);
    let mut off = rule(json!({"type": "schedule", "every": {"n": 1, "unit": "minutes"}}));
    off.enabled = false;
    assert!(due_events(&[off], &state, 10_000.0).is_empty());
    assert!(due_events(&[rule(json!({"type": "created"}))], &state, 10_000.0).is_empty());
}
