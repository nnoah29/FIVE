//! Règles planifiées : quand `Schedule`, `Daily` et `Weekly` sont à échéance.

use super::evaluate::EventKind;
use super::state::RulesState;
use super::time_calc::{civil, seconds, weekday};
use super::types_rule::{Rule, Trigger};
use crate::clock::local_timestamp;

/// Instant local d'une heure `HH:MM` le jour de `now`.
fn today_at(now: f64, at: &str) -> Option<f64> {
    let (hour, minute) = at.split_once(':')?;
    let (year, month, day) = civil(now);
    local_timestamp(year, month, day, hour.parse().ok()?, minute.parse().ok()?)
}

/// Indique si le déclencheur est à échéance : dernier passage connu, sinon jamais (la règle est d'abord mémorisée).
fn due(trigger: &Trigger, last: Option<f64>, now: f64) -> Option<EventKind> {
    let last = last?;
    match trigger {
        Trigger::Schedule { every } => (now - last >= seconds(every)).then_some(EventKind::Schedule),
        Trigger::Daily { at } => today_at(now, at).filter(|t| now >= *t && last < *t).map(|_| EventKind::Daily),
        Trigger::Weekly { day, at } => today_at(now, at).filter(|t| weekday(now) == *day && now >= *t && last < *t).map(|_| EventKind::Weekly),
        _ => None,
    }
}

/// Règles dont un déclencheur planifié est à échéance, avec le type d'événement.
pub fn due_events(rules: &[Rule], state: &RulesState, now: f64) -> Vec<(String, EventKind)> {
    let last = |rule: &Rule| state.last_run.get(&rule.id).copied();
    rules.iter().filter(|r| r.enabled).filter_map(|r| r.triggers.iter().find_map(|t| due(t, last(r), now)).map(|event| (r.id.clone(), event))).collect()
}
