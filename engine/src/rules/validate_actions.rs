//! Validation des actions : combinaisons contradictoires.

use super::types_action::{AfterUnzip, RuleAction as A};
use super::types_rule::Rule;
use super::validate::Problem;

/// Indique si l'action déplace le fichier vers un autre dossier.
fn moves(action: &A) -> bool {
    matches!(action, A::MoveTo { .. } | A::MoveToDated { .. } | A::MoveToByKind { .. } | A::MoveToByExtension { .. })
}

/// Indique si l'action met le fichier (ou l'archive d'origine) à la corbeille.
fn trashes(action: &A) -> bool {
    matches!(action, A::Trash | A::TrashDuplicates { .. } | A::ZipFolderThenTrash | A::Unzip { then: AfterUnzip::TrashArchive })
}

/// Signale deux déplacements, un déplacement avec une mise à la corbeille, ou une compression avec une extraction du même fichier.
pub fn check_contradictions(rule: &Rule, out: &mut Vec<Problem>) {
    let count = |test: fn(&A) -> bool| rule.actions.iter().filter(|a| test(a)).count();
    if count(moves) > 1 {
        out.push(Problem::new("contradiction", "Plusieurs déplacements dans la même règle : un fichier n'a qu'une destination."));
    }
    if count(moves) > 0 && count(trashes) > 0 {
        out.push(Problem::new("contradiction", "La règle déplace le fichier et le met aussi à la corbeille."));
    }
    let zips = rule.actions.iter().any(|a| matches!(a, A::Zip { .. } | A::ZipFolderThenTrash));
    if zips && rule.actions.iter().any(|a| matches!(a, A::Unzip { .. })) {
        out.push(Problem::new("contradiction", "La règle compresse et extrait le même fichier."));
    }
}
