//! Conditions sur les dates de modification, de création et d'accès.

use super::context::FileContext;
use super::time_calc::{civil, seconds, weekday};
use super::types_condition::ConditionTest as T;

/// Évalue une condition de date ; une date inconnue rend la condition fausse.
pub fn eval_dates(ctx: &FileContext, test: &T) -> Option<bool> {
    let (modified, created, accessed) = (ctx.modified, ctx.created, ctx.accessed);
    let age = |at: Option<f64>| at.map(|t| ctx.now - t);
    Some(match test {
        T::ModifiedBefore { at } => modified.is_some_and(|t| t < *at),
        T::ModifiedAfter { at } => modified.is_some_and(|t| t > *at),
        T::ModifiedWithin { duration } => age(modified).is_some_and(|a| a <= seconds(duration)),
        T::OlderThan { duration } => age(modified).is_some_and(|a| a > seconds(duration)),
        T::CreatedBefore { at } => created.is_some_and(|t| t < *at),
        T::CreatedAfter { at } => created.is_some_and(|t| t > *at),
        T::AccessedOlderThan { duration } => age(accessed).is_some_and(|a| a > seconds(duration)),
        T::ModifiedOnWeekday { days } => modified.is_some_and(|t| days.contains(&weekday(t))),
        T::ModifiedInMonth { months } => modified.is_some_and(|t| months.contains(&civil(t).1)),
        T::ModifiedInYear { years } => modified.is_some_and(|t| years.contains(&civil(t).0)),
        T::NeverOpened => accessed.zip(created).is_some_and(|(a, c)| (a - c).abs() < 1.0),
        _ => return None,
    })
}
