//! Informations coûteuses qu'une règle demande : on ne calcule que ce que les conditions et les modèles utilisent.

use super::types_action::{Keep, RuleAction as A};
use super::types_condition::ConditionTest as T;
use super::types_rule::Rule;
use super::validate_values::visit_tests;

/// Ce qu'il faut lire ou calculer pour évaluer des règles sur un fichier.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Needs {
    pub text: bool,
    pub image: bool,
    pub hash: bool,
    pub archive: bool,
    pub git: bool,
    pub origin: bool,
    pub duplicates: bool,
    pub children: bool,
}

/// Besoins d'une condition.
fn of_test(test: &T, needs: &mut Needs) {
    match test {
        T::TextContains { .. } | T::LineCount { .. } | T::FirstLine { .. } | T::HasShebang | T::Encoding { .. } => needs.text = true,
        T::ImageWidth { .. } | T::ImageHeight { .. } | T::Orientation { .. } | T::Megapixels { .. } | T::HasAlpha => needs.image = true,
        T::ArchiveIsReadable | T::ArchiveEntryCount { .. } | T::ArchiveContainsName { .. } | T::ArchiveIsEncrypted => needs.archive = true,
        T::DuplicateOfAnother { .. } | T::IsNewestOfDuplicates { .. } | T::IsOldestOfDuplicates { .. } | T::SameNameAs { .. } => needs.duplicates = true,
        T::GitStatus { .. } => needs.git = true,
        T::DownloadedFrom { .. } => needs.origin = true,
        T::ChildCount { .. } | T::FileCount { .. } | T::FolderTotalSize { .. } => needs.children = true,
        _ => {}
    }
}

/// Besoins d'une action : les variables de ses modèles et le choix des doublons à garder.
fn of_action(action: &A, needs: &mut Needs) {
    let text = serde_json::to_string(action).unwrap_or_default();
    needs.hash |= text.contains("{hash");
    needs.image |= text.contains("{width") || text.contains("{height");
    if let A::TrashDuplicates { keep: Keep::Newest | Keep::Oldest | Keep::ShortestName } = action {
        needs.duplicates = true;
    }
}

/// Réunit les besoins de toutes les règles données.
pub fn needs_of(rules: &[&Rule]) -> Needs {
    let mut needs = Needs::default();
    for rule in rules {
        for group in &rule.alternatives {
            visit_tests(&group.conditions, &mut |test| of_test(test, &mut needs));
        }
        rule.actions.iter().for_each(|action| of_action(action, &mut needs));
    }
    needs
}
