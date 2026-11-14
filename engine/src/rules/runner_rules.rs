//! Règles à appliquer : lues dans `regles.json`, sans les règles désactivées ni invalides.

use std::path::Path;

use super::status::{RuleStatus, rule_status};
use super::store::load;
use super::types_rule::Rule;
use super::validate::RootEnv;
use crate::root::Root;

/// Règles activées et valides (une règle en essai est comprise : elle ne fait que journaliser ce qu'elle ferait).
pub fn active_rules(root: &Root, dir: &Path) -> Vec<Rule> {
    let env = RootEnv(root);
    load(&dir.join("regles.json")).rules.into_iter().filter(|r| matches!(rule_status(r, &env), RuleStatus::Trial | RuleStatus::Active)).collect()
}
