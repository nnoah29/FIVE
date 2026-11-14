mod support;

use std::fs;
use std::sync::Arc;

use five_engine::jobs::registry::Jobs;
use five_engine::rules::run_types::{ActionResult, Outcome};
use five_engine::rules::runner::RulesService;
use five_engine::rules::runner_limits::limit_notifications;
use five_engine::rules::state::RulesState;
use five_engine::rules::store::{Stored, save};
use five_engine::rules::types_rule::Rule;
use serde_json::{Value, json};
use support::Sandbox;

fn rule(trigger: Value, watch: &str) -> Rule {
    serde_json::from_value(json!({
        "id": "r1", "name": "n", "trial": false, "watch": [{"path": watch}], "triggers": [trigger],
        "alternatives": [{"conditions": [{"test": {"type": "is_file"}}]}], "actions": [{"type": "add_to_favorites"}]
    }))
    .unwrap()
}

fn service(sandbox: &Sandbox, rules: Vec<Rule>) -> RulesService {
    fs::create_dir_all(sandbox.dir.join("Dl")).unwrap();
    let dir = sandbox.dir.join(".config/five");
    save(&dir.join("regles.json"), &Stored { rules, ..Stored::default() }).unwrap();
    RulesService::new(sandbox.root(), Arc::new(Jobs::new()), dir)
}

fn notice(action: &str, rule: &str) -> ActionResult {
    ActionResult { rule_id: rule.into(), path: "Dl/a".into(), action: action.into(), outcome: Outcome::Deferred, detail: String::new(), undo: Vec::new() }
}

#[test]
fn temporary_files_wait_until_they_are_renamed() {
    let sandbox = Sandbox::new();
    let mut svc = service(&sandbox, vec![rule(json!({"type": "created"}), "Dl")]);
    svc.tick(100.0);
    sandbox.write("Dl/film.mp4.part", "x");
    assert!(svc.tick(105.0).results.is_empty());
    fs::rename(sandbox.dir.join("Dl/film.mp4.part"), sandbox.dir.join("Dl/film.mp4")).unwrap();
    assert_eq!(svc.tick(110.0).results.len(), 1);
}

#[test]
fn notifications_are_capped_per_day_and_notify_only_once_is_once_per_pass() {
    let mut state = RulesState::default();
    let many: Vec<_> = (0..1002).map(|_| notice("notify", "r")).collect();
    let out = limit_notifications(&mut state, many, 1_700_000_000.0);
    assert_eq!((out.iter().filter(|r| r.outcome == Outcome::Deferred).count(), out.last().unwrap().outcome), (1000, Outcome::Skipped));
    let mut fresh = RulesState::default();
    let once = limit_notifications(
        &mut fresh,
        vec![notice("notify_only_once", "a"), notice("notify_only_once", "a"), notice("notify_only_once", "b")],
        1_700_000_000.0,
    );
    assert_eq!(once.iter().map(|r| r.outcome).collect::<Vec<_>>(), [Outcome::Deferred, Outcome::Skipped, Outcome::Deferred]);
}
