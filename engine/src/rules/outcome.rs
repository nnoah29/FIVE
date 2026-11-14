//! Résultat d'une évaluation : actions prévues, règles qui ont correspondu, règles ou actions écartées.

use super::types_action::RuleAction;
use super::types_rule::ConflictPolicy;

/// Action prévue pour un fichier : l'action de la règle et le texte calculé (nouveau nom, dossier, message).
#[derive(Debug, Clone, PartialEq)]
pub struct PlannedAction {
    pub rule_id: String,
    pub path: String,
    pub action: RuleAction,
    pub text: Option<String>,
    pub dry_run: bool,
    pub on_conflict: ConflictPolicy,
}

/// Action ou règle écartée, avec la raison affichée à l'utilisateur.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Skipped {
    pub rule_id: String,
    pub reason: String,
}

/// Sortie de l'évaluateur pour un fichier.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct RuleOutcome {
    pub actions: Vec<PlannedAction>,
    pub matched: Vec<String>,
    pub skipped: Vec<Skipped>,
}
