mod support;

use std::fs;
use std::sync::Arc;

use five_engine::jobs::registry::Jobs;
use five_engine::rules::journal::journal_path;
use five_engine::rules::run_types::Outcome;
use five_engine::rules::runner::RulesService;
use five_engine::rules::store::{Stored, save};
use five_engine::rules::types_rule::Rule;
use five_engine::rules::undo::undo_last;
use serde_json::json;
use support::Sandbox;

fn rule(trigger: serde_json::Value, actions: serde_json::Value, trial: bool) -> Rule {
    serde_json::from_value(json!({
        "id": "r1", "name": "PDF", "trial": trial,
        "watch": [{"path": "Dl"}], "triggers": [trigger],
        "alternatives": [{"conditions": [{"test": {"type": "extension", "in_list": ["pdf"]}}]}],
        "actions": actions
    }))
    .unwrap()
}

fn service(sandbox: &Sandbox, rules: Vec<Rule>) -> RulesService {
    for folder in ["Dl", "Docs"] {
        fs::create_dir_all(sandbox.dir.join(folder)).unwrap();
    }
    let dir = sandbox.dir.join(".config/five");
    save(&dir.join("regles.json"), &Stored { rules, ..Stored::default() }).unwrap();
    RulesService::new(sandbox.root(), Arc::new(Jobs::new()), dir)
}

#[test]
fn a_rule_stops_after_one_hundred_files_per_pass() {
    let sandbox = Sandbox::new();
    let mut svc = service(&sandbox, vec![rule(json!({"type": "created"}), json!([{"type": "add_to_favorites"}]), false)]);
    svc.tick(100.0);
    for n in 0..105 {
        sandbox.write(&format!("Dl/f{n:03}.pdf"), "x");
    }
    let results = svc.tick(105.0).results;
    let done = results.iter().filter(|r| r.outcome == Outcome::Deferred).count();
    let limited = results.iter().filter(|r| r.outcome == Outcome::Skipped).count();
    assert_eq!((done, limited), (100, 5));
}

#[test]
fn run_now_preview_and_undo_work_on_demand() {
    let sandbox = Sandbox::new();
    let rule = rule(json!({"type": "manual"}), json!([{"type": "move_to", "folder": "Docs"}]), false);
    let mut svc = service(&sandbox, vec![rule.clone()]);
    sandbox.write("Dl/a.pdf", "x");
    let planned = svc.preview(&rule, None, 100.0);
    assert!(planned.len() == 1 && planned[0].dry_run && sandbox.dir.join("Dl/a.pdf").exists());
    let report = svc.run_now(Some("r1"), None, 100.0);
    assert_eq!(report.results[0].outcome, Outcome::Done);
    assert!(sandbox.dir.join("Docs/a.pdf").exists());
    undo_last(&sandbox.root(), &journal_path(&sandbox.dir.join(".config/five"))).unwrap();
    assert!(sandbox.dir.join("Dl/a.pdf").exists() && !sandbox.dir.join("Docs/a.pdf").exists());
}
