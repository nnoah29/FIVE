//! Plan d'une règle pour un fichier : actions dans l'ordre, noms calculés, actions écartées.

use super::context::FileContext;
use super::outcome::{PlannedAction, Skipped};
use super::plan_text::{is_rename, new_name, target_text};
use super::time_calc::seconds;
use super::types_action::{AfterUnzip, RuleAction as A};
use super::types_rule::Rule;

/// Ce qu'une règle prévoit pour un fichier.
#[derive(Debug, Default)]
pub struct RulePlan {
    pub actions: Vec<PlannedAction>,
    pub skipped: Vec<Skipped>,
    pub renamed: bool,
    pub left: bool,
    pub stop: bool,
}

/// Indique si l'action fait quitter le fichier de sa place : déplacement ou mise à la corbeille.
fn leaves(action: &A) -> bool {
    matches!(
        action,
        A::MoveTo { .. }
            | A::MoveToDated { .. }
            | A::MoveToByKind { .. }
            | A::MoveToByExtension { .. }
            | A::Trash
            | A::TrashDuplicates { .. }
            | A::ZipFolderThenTrash
            | A::Unzip { then: AfterUnzip::TrashArchive }
    )
}

/// Prévoit une action : renommage (sur le nom courant) ou texte calculé ; `Err` avec la raison si elle ne s'applique pas.
fn plan_one(rule: &Rule, ctx: &FileContext, action: &A, current: &mut String, dry_run: bool) -> Result<PlannedAction, String> {
    let text = if is_rename(action) {
        let name = new_name(action, ctx, current)?;
        if name == *current {
            return Err("renommage sans effet".into());
        }
        current.clone_from(&name);
        Some(name)
    } else {
        target_text(action, ctx)?
    };
    Ok(PlannedAction { rule_id: rule.id.clone(), path: ctx.path.clone(), action: action.clone(), text, dry_run, on_conflict: rule.on_conflict })
}

/// Indique si le fichier est assez stable pour que l'action `WaitStable` laisse passer la suite.
fn stable(ctx: &FileContext, wait: &super::types_matcher::DurationSpec) -> bool {
    ctx.stable_for.is_some_and(|s| s >= seconds(wait))
}

/// Prévoit toutes les actions d'une règle ; `prior_rename` signale qu'une règle précédente a déjà renommé le fichier.
pub fn plan_rule(rule: &Rule, ctx: &FileContext, prior_rename: bool) -> RulePlan {
    let dry_run = rule.trial || rule.actions.contains(&A::LogOnly);
    let (mut plan, mut current) = (RulePlan::default(), ctx.name.clone());
    let skip = |plan: &mut RulePlan, reason: &str| plan.skipped.push(Skipped { rule_id: rule.id.clone(), reason: reason.to_string() });
    for action in &rule.actions {
        match action {
            A::StopProcessing => plan.stop = true,
            A::SkipRemainingActions => break,
            A::LogOnly => {}
            A::WaitStable { wait } if !stable(ctx, wait) => {
                return {
                    skip(&mut plan, "en attente : le fichier n'est pas encore stable");
                    plan
                };
            }
            A::WaitStable { .. } => {}
            _ if is_rename(action) && prior_rename => skip(&mut plan, "déjà renommé par une règle précédente"),
            _ => match plan_one(rule, ctx, action, &mut current, dry_run) {
                Ok(planned) => {
                    plan.renamed |= is_rename(action);
                    plan.left |= leaves(action);
                    plan.actions.push(planned);
                }
                Err(reason) => skip(&mut plan, &reason),
            },
        }
    }
    plan
}
