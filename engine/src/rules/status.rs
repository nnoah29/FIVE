//! Statut calculé d'une règle : désactivée, en essai, active ou invalide.

use super::types_rule::Rule;
use super::validate::{Env, validate_rule};

/// Statut affiché dans la liste des règles.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuleStatus {
    Disabled,
    Trial,
    Active,
    Invalid(String),
}

impl RuleStatus {
    /// Nom du statut tel que l'API le renvoie.
    pub fn as_str(&self) -> &'static str {
        match self {
            RuleStatus::Disabled => "disabled",
            RuleStatus::Trial => "trial",
            RuleStatus::Active => "active",
            RuleStatus::Invalid(_) => "invalid",
        }
    }
}

/// Statut d'une règle : invalide (avec la première raison) avant tout, puis désactivée, en essai ou active.
pub fn rule_status(rule: &Rule, env: &dyn Env) -> RuleStatus {
    if let Some(problem) = validate_rule(rule, env).into_iter().next() {
        return RuleStatus::Invalid(problem.message);
    }
    match (rule.enabled, rule.trial) {
        (false, _) => RuleStatus::Disabled,
        (true, true) => RuleStatus::Trial,
        (true, false) => RuleStatus::Active,
    }
}
