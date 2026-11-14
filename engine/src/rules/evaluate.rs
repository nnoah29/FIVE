//! Évaluation de toutes les règles pour un fichier et un événement.

use super::context::FileContext;
use super::eval::rule_matches;
use super::outcome::{RuleOutcome, Skipped};
use super::plan::plan_rule;
use super::temp_names::is_temporary;
use super::time_calc::seconds;
use super::types_rule::{Rule, Trigger, Watch};

/// Événement qui provoque l'évaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventKind {
    Created,
    Modified,
    MovedIn,
    Stable,
    Schedule,
    Daily,
    Weekly,
    OnStart,
    OnMount,
    Manual,
}

/// Indique si le fichier est dans le dossier surveillé, selon la récursivité, la profondeur et les éléments cachés.
pub fn watch_contains(watch: &Watch, path: &str) -> bool {
    let base = watch.path.trim_matches('/');
    let rel = if base.is_empty() { Some(path) } else { path.strip_prefix(&format!("{base}/")) };
    let Some(rel) = rel.filter(|r| !r.is_empty()) else { return false };
    let depth = rel.matches('/').count() as u32;
    let deep_enough = if watch.recursive { watch.max_depth.is_none_or(|max| depth <= max) } else { depth == 0 };
    deep_enough && !(watch.ignore_hidden && rel.split('/').any(|part| part.starts_with('.')))
}

/// Indique si le déclencheur correspond à l'événement ; `Stable` exige que la taille soit restée fixe assez longtemps.
fn trigger_matches(trigger: &Trigger, event: EventKind, ctx: &FileContext) -> bool {
    match (trigger, event) {
        (Trigger::Created, EventKind::Created) | (Trigger::Modified, EventKind::Modified) | (Trigger::MovedIn, EventKind::MovedIn) => true,
        (Trigger::Stable { wait }, EventKind::Stable) => ctx.stable_for.is_some_and(|s| s >= seconds(wait)),
        (Trigger::Schedule { .. }, EventKind::Schedule) | (Trigger::Daily { .. }, EventKind::Daily) | (Trigger::Weekly { .. }, EventKind::Weekly) => true,
        (Trigger::OnStart, EventKind::OnStart) | (Trigger::OnMount { .. }, EventKind::OnMount) | (Trigger::Manual, EventKind::Manual) => true,
        _ => false,
    }
}

/// Indique si la règle est activée, surveille ce fichier et se déclenche pour cet événement (« Appliquer maintenant » vaut pour toutes).
fn applies(rule: &Rule, ctx: &FileContext, event: EventKind) -> bool {
    let watched = rule.watch.iter().any(|w| watch_contains(w, &ctx.path));
    let triggered = event == EventKind::Manual || rule.triggers.iter().any(|t| trigger_matches(t, event, ctx));
    rule.enabled && watched && triggered
}

/// Évalue les règles dans l'ordre pour un fichier (les règles invalides sont retirées par l'appelant) : actions prévues, règles correspondantes, actions écartées.
pub fn evaluate(rules: &[Rule], ctx: &FileContext, event: EventKind) -> RuleOutcome {
    let mut outcome = RuleOutcome::default();
    if event != EventKind::Manual && is_temporary(ctx) {
        return outcome;
    }
    let mut renamed = false;
    for rule in rules.iter().filter(|r| applies(r, ctx, event) && rule_matches(r, ctx)) {
        let plan = plan_rule(rule, ctx, renamed);
        renamed |= plan.renamed;
        outcome.matched.push(rule.id.clone());
        outcome.actions.extend(plan.actions);
        outcome.skipped.extend(plan.skipped);
        if plan.stop || plan.left {
            let reason = if plan.stop { "traitement arrêté par la règle" } else { "fichier déplacé ou mis à la corbeille : règles suivantes ignorées" };
            outcome.skipped.push(Skipped { rule_id: rule.id.clone(), reason: reason.into() });
            break;
        }
    }
    outcome
}
