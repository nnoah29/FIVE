//! Validation des dossiers d'une règle : surveillés, destinations, boucles.

use super::types_action::RuleAction as A;
use super::types_condition::ConditionTest as T;
use super::types_rule::Rule;
use super::validate::{Env, FolderStatus, Problem};

/// Début d'un chemin sans variable : les segments avant le premier qui contient `{`.
fn static_prefix(path: &str) -> String {
    path.split('/').take_while(|part| !part.contains('{')).collect::<Vec<_>>().join("/")
}

/// Dossiers de destination des actions de déplacement et de copie, sans leurs sous-dossiers variables.
fn destinations(rule: &Rule) -> Vec<String> {
    rule.actions
        .iter()
        .filter_map(|action| match action {
            A::MoveTo { folder }
            | A::CopyTo { folder }
            | A::MoveToDated { folder, .. }
            | A::MoveToByKind { base: folder }
            | A::MoveToByExtension { base: folder } => Some(static_prefix(folder.trim_matches('/'))),
            _ => None,
        })
        .collect()
}

/// Phrase de refus pour un dossier inutilisable ; `None` s'il est correct.
fn status_problem(status: FolderStatus, role: &str, path: &str) -> Option<Problem> {
    let reason = match status {
        FolderStatus::Ok => return None,
        FolderStatus::Missing => "n'existe pas",
        FolderStatus::OutsideRoot => "est hors de la racine exposée",
        FolderStatus::InTrash => "est dans la corbeille",
    };
    Some(Problem::new("folder", format!("Le {role} « /{path} » {reason}.")))
}

/// Vérifie les dossiers surveillés (la corbeille n'est permise que pour vider la corbeille) et les destinations.
pub fn check_folders(rule: &Rule, env: &dyn Env, out: &mut Vec<Problem>) {
    let only_trash_cleanup = rule.actions.iter().all(|a| matches!(a, A::EmptyTrashOlderThan { .. }));
    for watch in &rule.watch {
        let status = env.folder_status(watch.path.trim_matches('/'));
        if let Some(problem) = status_problem(status, "dossier surveillé", &watch.path).filter(|_| !(status == FolderStatus::InTrash && only_trash_cleanup)) {
            out.push(problem);
        }
    }
    for folder in destinations(rule) {
        out.extend(status_problem(env.folder_status(&folder), "dossier de destination", &folder));
    }
}

/// Indique si chaque alternative exclut explicitement (condition « dans le dossier » inversée) le dossier `destination`.
fn excluded(rule: &Rule, destination: &str) -> bool {
    let covers = |path: &str| {
        let path = path.trim_matches('/');
        !path.is_empty() && (destination == path || destination.starts_with(&format!("{path}/")))
    };
    rule.alternatives.iter().all(|alt| alt.conditions.iter().any(|c| c.negated && matches!(&c.test, T::InFolder { path, .. } if covers(path))))
}

/// Signale une destination à l'intérieur d'un dossier surveillé récursivement : la règle se re-déclencherait sur ses propres fichiers.
pub fn check_loops(rule: &Rule, out: &mut Vec<Problem>) {
    for destination in destinations(rule) {
        for watch in rule.watch.iter().filter(|w| w.recursive) {
            let base = watch.path.trim_matches('/');
            let inside = base.is_empty() || destination == base || destination.starts_with(&format!("{base}/"));
            if inside && !excluded(rule, &destination) {
                out.push(Problem::new(
                    "loop",
                    format!(
                        "La destination « /{destination} » est dans un dossier surveillé : ajoutez la condition « pas dans ce dossier » pour éviter la boucle."
                    ),
                ));
            }
        }
    }
}
