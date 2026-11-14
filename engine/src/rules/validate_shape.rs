//! Validation de la forme d'une règle : éléments obligatoires, plafonds, déclencheurs.

use super::time_calc::seconds;
use super::types_action::RuleAction as A;
use super::types_matcher::DurationSpec;
use super::types_rule::{Rule, Trigger};
use super::validate::Problem;

const MAX_ALTERNATIVES: usize = 10;
const MAX_CONDITIONS: usize = 30;
const MAX_ACTIONS: usize = 20;
const MIN_SCHEDULE: f64 = 60.0;
const MIN_STABLE: f64 = 2.0;

/// Signale ce qui manque : nom, dossier surveillé, déclencheur, conditions, action.
pub fn check_presence(rule: &Rule, out: &mut Vec<Problem>) {
    let mut missing = |code: &'static str, text: &str| out.push(Problem::new(code, text));
    if rule.name.trim().is_empty() {
        missing("name", "La règle n'a pas de nom.");
    }
    if rule.watch.is_empty() {
        missing("watch", "Choisissez au moins un dossier à surveiller.");
    }
    if rule.triggers.is_empty() {
        missing("trigger", "Choisissez au moins un déclencheur.");
    }
    if rule.alternatives.is_empty() || rule.alternatives.iter().any(|g| g.conditions.is_empty()) {
        missing("conditions", "Chaque alternative doit avoir au moins une condition.");
    }
    if rule.actions.is_empty() {
        missing("actions", "Choisissez au moins une action.");
    }
}

/// Signale les plafonds dépassés : alternatives, conditions par alternative, actions.
pub fn check_limits(rule: &Rule, out: &mut Vec<Problem>) {
    if rule.alternatives.len() > MAX_ALTERNATIVES {
        out.push(Problem::new("limit", format!("Plus de {MAX_ALTERNATIVES} alternatives.")));
    }
    if rule.alternatives.iter().any(|g| g.conditions.len() > MAX_CONDITIONS) {
        out.push(Problem::new("limit", format!("Plus de {MAX_CONDITIONS} conditions dans une alternative.")));
    }
    if rule.actions.len() > MAX_ACTIONS {
        out.push(Problem::new("limit", format!("Plus de {MAX_ACTIONS} actions.")));
    }
}

/// Indique si le texte est une heure `HH:MM` valide.
fn valid_time(at: &str) -> bool {
    at.split_once(':').is_some_and(|(h, m)| h.len() == 2 && m.len() == 2 && h.parse::<u8>().is_ok_and(|h| h < 24) && m.parse::<u8>().is_ok_and(|m| m < 60))
}

/// Signale un intervalle trop court, une durée de stabilité trop courte ou une heure invalide.
pub fn check_triggers(rule: &Rule, out: &mut Vec<Problem>) {
    let too_short = |d: &DurationSpec, min: f64| seconds(d) < min;
    for trigger in &rule.triggers {
        match trigger {
            Trigger::Schedule { every } if too_short(every, MIN_SCHEDULE) => {
                out.push(Problem::new("schedule", "Un intervalle ne peut pas être inférieur à 1 minute."))
            }
            Trigger::Stable { wait } if too_short(wait, MIN_STABLE) => {
                out.push(Problem::new("stable", "La durée de stabilité ne peut pas être inférieure à 2 secondes."))
            }
            Trigger::Daily { at } | Trigger::Weekly { at, .. } if !valid_time(at) => {
                out.push(Problem::new("time", format!("Heure « {at} » invalide : écrire HH:MM.")))
            }
            _ => {}
        }
    }
    for action in &rule.actions {
        if matches!(action, A::WaitStable { wait } if too_short(wait, MIN_STABLE)) {
            out.push(Problem::new("stable", "L'attente de stabilité ne peut pas être inférieure à 2 secondes."));
        }
    }
}
