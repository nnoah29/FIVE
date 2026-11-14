//! Validation des valeurs d'une règle : expressions régulières, motifs, modèles de noms, longueurs.

use regex::RegexBuilder;

use super::template::{VARIABLES, template_variables};
use super::types_action::RuleAction as A;
use super::types_condition::{Condition, ConditionTest as T};
use super::types_matcher::{MatchOp, TextMatcher};
use super::types_rule::Rule;
use super::validate::Problem;

const MAX_VALUE: usize = 1_000;
const MAX_STARS: usize = 8;

/// Parcourt tous les tests d'une liste de conditions, y compris ceux de `AllOf`, `AnyOf` et `NoneOf`.
pub fn visit_tests(conditions: &[Condition], visit: &mut impl FnMut(&T)) {
    for condition in conditions {
        visit(&condition.test);
        if let T::AllOf { tests } | T::AnyOf { tests } | T::NoneOf { tests } = &condition.test {
            visit_tests(tests, visit);
        }
    }
}

/// Raison pour laquelle une comparaison de texte est refusée : trop longue, expression invalide, motif trop complexe.
fn matcher_problem(matcher: &TextMatcher) -> Option<String> {
    let values = std::iter::once(&matcher.value).chain(matcher.values.iter());
    if values.clone().any(|v| v.len() > MAX_VALUE) {
        return Some(format!("valeur de plus de {MAX_VALUE} caractères"));
    }
    match matcher.op {
        MatchOp::Regex => RegexBuilder::new(&matcher.value).size_limit(1 << 20).build().err().map(|e| format!("expression régulière invalide : {e}")),
        MatchOp::Glob if matcher.value.matches('*').count() > MAX_STARS => Some(format!("motif trop complexe (plus de {MAX_STARS} « * »)")),
        _ => None,
    }
}

/// Signale les comparaisons de texte invalides de toutes les conditions.
pub fn check_matchers(rule: &Rule, out: &mut Vec<Problem>) {
    for group in &rule.alternatives {
        visit_tests(&group.conditions, &mut |test| {
            let matcher = match test {
                T::Name { matcher }
                | T::Stem { matcher }
                | T::Path { matcher }
                | T::ParentName { matcher }
                | T::Mime { matcher }
                | T::TextContains { matcher }
                | T::FirstLine { matcher } => Some(matcher),
                T::ArchiveContainsName { matcher } | T::DownloadedFrom { matcher } => Some(matcher),
                _ => None,
            };
            if let Some(reason) = matcher.and_then(matcher_problem) {
                out.push(Problem::new("matcher", reason));
            }
        });
    }
}

/// Modèles d'une action : nom, chemin ou message dans lesquels des variables `{…}` sont permises.
fn templates_of(action: &A) -> Vec<&str> {
    match action {
        A::Rename { template } | A::CreateFolder { template } => vec![template],
        A::AddPrefix { text } | A::AddSuffix { text } => vec![text],
        A::MoveTo { folder } | A::CopyTo { folder } | A::MoveToByKind { base: folder } | A::MoveToByExtension { base: folder } => vec![folder],
        A::MoveToDated { folder, pattern } => vec![folder, pattern],
        A::Zip { name_template: Some(template), .. } => vec![template],
        A::Notify { message_template } => vec![message_template],
        _ => Vec::new(),
    }
}

/// Signale les modèles dont une accolade est mal fermée ou dont une variable est inconnue.
pub fn check_templates(rule: &Rule, out: &mut Vec<Problem>) {
    for template in rule.actions.iter().flat_map(templates_of) {
        match template_variables(template) {
            Err(reason) => out.push(Problem::new("template", format!("modèle « {template} » : {reason}"))),
            Ok(names) => names
                .iter()
                .filter(|n| !VARIABLES.contains(&n.as_str()))
                .for_each(|n| out.push(Problem::new("template", format!("variable « {n} » inconnue dans « {template} »")))),
        }
    }
}
