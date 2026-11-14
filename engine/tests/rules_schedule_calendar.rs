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
fn a_daily_rule_runs_once_after_its_time() {
    let daily = json!({"type": "daily", "at": "08:30"});
    let day_start = 1_705_276_800.0;
    assert!(due(daily.clone(), Some(day_start), day_start + 8.0 * 3600.0).is_empty());
    assert_eq!(due(daily.clone(), Some(day_start), day_start + 9.0 * 3600.0).len(), 1);
    assert!(due(daily, Some(day_start + 8.6 * 3600.0), day_start + 9.0 * 3600.0).is_empty());
}

#[test]
fn a_weekly_rule_only_runs_on_its_day() {
    let monday = 1_705_276_800.0;
    let weekly = |day: &str| json!({"type": "weekly", "day": day, "at": "08:30"});
    assert_eq!(due(weekly("mon"), Some(monday - 86_400.0), monday + 9.0 * 3600.0).len(), 1);
    assert!(due(weekly("tue"), Some(monday - 86_400.0), monday + 9.0 * 3600.0).is_empty());
}
