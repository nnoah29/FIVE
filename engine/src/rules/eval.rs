//! Évaluation d'une règle sur un fichier : conditions, groupes ET, alternatives OU, inversions.

use super::context::FileContext;
use super::eval_content::{eval_archive, eval_image, eval_media, eval_text};
use super::eval_dates::eval_dates;
use super::eval_misc::{eval_duplicates, eval_interface, eval_origin, eval_system};
use super::eval_name::{eval_name, eval_path};
use super::eval_nature::{eval_kind, eval_nature, eval_size};
use super::types_condition::{Condition, ConditionGroup, ConditionTest as T};
use super::types_rule::Rule;

/// Évalue `AllOf`, `AnyOf` et `NoneOf` ; `None` pour toute autre condition.
fn eval_compose(ctx: &FileContext, test: &T) -> Option<bool> {
    Some(match test {
        T::AllOf { tests } => tests.iter().all(|c| condition_holds(ctx, c)),
        T::AnyOf { tests } => tests.iter().any(|c| condition_holds(ctx, c)),
        T::NoneOf { tests } => !tests.iter().any(|c| condition_holds(ctx, c)),
        _ => return None,
    })
}

/// Cherche la famille qui sait évaluer le test ; `None` si aucune ne le connaît.
fn dispatch(ctx: &FileContext, test: &T) -> Option<bool> {
    eval_name(ctx, test)
        .or_else(|| eval_path(ctx, test))
        .or_else(|| eval_nature(ctx, test))
        .or_else(|| eval_kind(ctx, test))
        .or_else(|| eval_size(ctx, test))
        .or_else(|| eval_dates(ctx, test))
        .or_else(|| eval_text(ctx, test))
        .or_else(|| eval_image(ctx, test))
        .or_else(|| eval_media(ctx, test))
        .or_else(|| eval_archive(ctx, test))
        .or_else(|| eval_duplicates(ctx, test))
        .or_else(|| eval_origin(ctx, test))
        .or_else(|| eval_system(ctx, test))
        .or_else(|| eval_interface(ctx, test))
        .or_else(|| eval_compose(ctx, test))
}

/// Indique si le moteur sait évaluer ce test (vrai pour toutes les variantes du catalogue).
pub fn is_implemented(test: &T) -> bool {
    dispatch(&FileContext::default(), test).is_some()
}

/// Valeur d'une condition, inversion comprise ; un test inconnu est faux.
pub fn condition_holds(ctx: &FileContext, condition: &Condition) -> bool {
    dispatch(ctx, &condition.test).unwrap_or(false) != condition.negated
}

/// Indique si la règle correspond au fichier : au moins une alternative a toutes ses conditions vraies.
pub fn rule_matches(rule: &Rule, ctx: &FileContext) -> bool {
    let group_holds = |group: &ConditionGroup| !group.conditions.is_empty() && group.conditions.iter().all(|c| condition_holds(ctx, c));
    rule.alternatives.iter().any(group_holds)
}
