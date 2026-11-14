mod support;

use five_engine::rules::eval::is_implemented;
use five_engine::rules::types_action::RuleAction;
use five_engine::rules::types_condition::ConditionTest;
use five_engine::rules::types_rule::Trigger;
use support::catalog_samples::{ACTIONS, CONDITIONS, TRIGGERS};

#[test]
fn every_condition_of_the_catalog_is_readable_and_implemented() {
    let tests: Vec<ConditionTest> = serde_json::from_str(CONDITIONS).unwrap();
    assert_eq!(tests.len(), 74);
    assert!(tests.iter().all(is_implemented));
}

#[test]
fn every_action_of_the_catalog_is_readable_and_round_trips() {
    let actions: Vec<RuleAction> = serde_json::from_str(ACTIONS).unwrap();
    assert_eq!(actions.len(), 40);
    let again: Vec<RuleAction> = serde_json::from_value(serde_json::to_value(&actions).unwrap()).unwrap();
    assert_eq!(actions, again);
}

#[test]
fn every_trigger_of_the_catalog_is_readable() {
    let triggers: Vec<Trigger> = serde_json::from_str(TRIGGERS).unwrap();
    assert_eq!(triggers.len(), 10);
}

#[test]
fn an_unknown_variant_is_refused_instead_of_guessed() {
    assert!(serde_json::from_str::<ConditionTest>(r#"{"type": "est_belle"}"#).is_err());
    assert!(serde_json::from_str::<RuleAction>(r#"{"type": "executer_script", "commande": "rm"}"#).is_err());
}
