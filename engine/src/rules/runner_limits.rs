//! Plafonds de notifications : 1 000 par jour, une seule par règle et par passe pour `notify_only_once`.

use std::collections::HashSet;

use super::run_types::{ActionResult, Outcome};
use super::state::RulesState;
use super::time_calc::format_date;

const MAX_NOTIFICATIONS: u32 = 1_000;

/// Écarte les notifications au-delà du plafond du jour ou répétées ; le compteur du jour est conservé dans l'état.
pub fn limit_notifications(state: &mut RulesState, results: Vec<ActionResult>, now: f64) -> Vec<ActionResult> {
    let day = format_date(now, "AAAA-MM-JJ");
    if state.notify_day != day {
        (state.notify_day, state.notify_count) = (day, 0);
    }
    let mut once: HashSet<String> = HashSet::new();
    results
        .into_iter()
        .map(|mut result| {
            let notify = result.outcome == Outcome::Deferred && result.action.starts_with("notify");
            let repeated = result.action == "notify_only_once" && !once.insert(result.rule_id.clone());
            if notify && (repeated || state.notify_count >= MAX_NOTIFICATIONS) {
                (result.outcome, result.detail) =
                    (Outcome::Skipped, if repeated { "déjà notifié pour cette passe".into() } else { "plafond quotidien de notifications atteint".into() });
            } else if notify {
                state.notify_count += 1;
            }
            result
        })
        .collect()
}
