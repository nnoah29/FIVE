mod support;

use std::fs;

use five_engine::rules::run_types::Outcome;
use serde_json::json;
use support::Sandbox;
use support::rules_run::{any_file, run};

#[test]
fn rename_changes_the_name_in_place_and_chains_on_the_new_path() {
    let sandbox = Sandbox::new();
    sandbox.write("Dl/Mon Fichier.TXT", "x");
    let results = run(&sandbox, &any_file(json!([{"type": "slugify"}, {"type": "add_prefix", "text": "a-"}])), "Dl/Mon Fichier.TXT");
    assert!(results.iter().all(|r| r.outcome == Outcome::Done), "{results:?}");
    assert!(sandbox.dir.join("Dl/a-mon-fichier.txt").is_file() && !sandbox.dir.join("Dl/Mon Fichier.TXT").exists());
    assert_eq!(results[1].detail, "Dl/a-mon-fichier.txt");
}

#[test]
fn an_existing_name_is_never_overwritten_by_default() {
    let sandbox = Sandbox::new();
    sandbox.write("Dl/a b.txt", "nouveau");
    sandbox.write("Dl/a-b.txt", "ancien");
    run(&sandbox, &any_file(json!([{"type": "slugify"}])), "Dl/a b.txt");
    assert_eq!(fs::read_to_string(sandbox.dir.join("Dl/a-b.txt")).unwrap(), "ancien");
    assert_eq!(fs::read_to_string(sandbox.dir.join("Dl/a-b (2).txt")).unwrap(), "nouveau");
}

#[test]
fn the_skip_and_ask_policies_leave_the_file_alone() {
    let sandbox = Sandbox::new();
    sandbox.write("Dl/a b.txt", "1");
    sandbox.write("Dl/a-b.txt", "2");
    for (policy, expected) in [("skip", Outcome::Skipped), ("ask", Outcome::Ask)] {
        let mut rule = any_file(json!([{"type": "slugify"}]));
        rule.on_conflict = serde_json::from_value(json!(policy)).unwrap();
        assert_eq!(run(&sandbox, &rule, "Dl/a b.txt")[0].outcome, expected);
        assert!(sandbox.dir.join("Dl/a b.txt").exists());
    }
}

#[test]
fn a_trial_rule_only_says_what_it_would_do() {
    let sandbox = Sandbox::new();
    sandbox.write("Dl/a b.txt", "1");
    let mut rule = any_file(json!([{"type": "slugify"}]));
    rule.trial = true;
    let results = run(&sandbox, &rule, "Dl/a b.txt");
    assert_eq!((results[0].outcome, results[0].detail.as_str()), (Outcome::DryRun, "a-b.txt"));
    assert!(sandbox.dir.join("Dl/a b.txt").exists());
}

#[test]
fn hiding_adds_or_removes_the_leading_dot() {
    let sandbox = Sandbox::new();
    sandbox.write("Dl/secret.txt", "1");
    run(&sandbox, &any_file(json!([{"type": "hide", "value": true}])), "Dl/secret.txt");
    assert!(sandbox.dir.join("Dl/.secret.txt").is_file());
    let mut show = any_file(json!([{"type": "hide", "value": false}]));
    show.watch[0].ignore_hidden = false;
    run(&sandbox, &show, "Dl/.secret.txt");
    assert!(sandbox.dir.join("Dl/secret.txt").is_file());
}
