//! Validation d'une règle à l'enregistrement : erreurs explicites en français.

use super::types_rule::Rule;
use super::validate_actions::check_contradictions;
use super::validate_folders::{check_folders, check_loops};
use super::validate_shape::{check_limits, check_presence, check_triggers};
use super::validate_values::{check_matchers, check_templates};
use crate::root::Root;

const MAX_RULES: usize = 200;

/// État d'un dossier référencé par une règle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FolderStatus {
    Ok,
    Missing,
    OutsideRoot,
    InTrash,
}

/// Ce que la validation demande au système de fichiers.
pub trait Env {
    /// État du dossier `path`, relatif à la racine.
    fn folder_status(&self, path: &str) -> FolderStatus;
}

/// Problème trouvé : un code stable et une phrase pour l'utilisateur.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Problem {
    pub code: &'static str,
    pub message: String,
}

impl Problem {
    /// Construit un problème.
    pub fn new(code: &'static str, message: impl Into<String>) -> Problem {
        Problem { code, message: message.into() }
    }
}

/// Environnement réel : la racine exposée.
pub struct RootEnv<'a>(pub &'a Root);

impl Env for RootEnv<'_> {
    /// Résout le dossier sous la racine : 403 hors racine, absent ou fichier, corbeille.
    fn folder_status(&self, path: &str) -> FolderStatus {
        match self.0.resolve(path) {
            Err(refus) if refus.code == 403 => FolderStatus::OutsideRoot,
            Err(_) => FolderStatus::Missing,
            Ok(full) if !full.is_dir() => FolderStatus::Missing,
            Ok(full) if self.0.in_trash(&full.join("x")) => FolderStatus::InTrash,
            Ok(_) => FolderStatus::Ok,
        }
    }
}

/// Valide une règle ; la liste est vide si elle est correcte.
pub fn validate_rule(rule: &Rule, env: &dyn Env) -> Vec<Problem> {
    let mut out = Vec::new();
    check_presence(rule, &mut out);
    check_limits(rule, &mut out);
    check_triggers(rule, &mut out);
    check_matchers(rule, &mut out);
    check_templates(rule, &mut out);
    check_contradictions(rule, &mut out);
    check_folders(rule, env, &mut out);
    check_loops(rule, &mut out);
    out
}

/// Valide l'ensemble : au plus 200 règles, identifiants non vides et uniques.
pub fn validate_set(rules: &[Rule]) -> Vec<Problem> {
    let mut out = Vec::new();
    if rules.len() > MAX_RULES {
        out.push(Problem::new("limit", format!("Plus de {MAX_RULES} règles.")));
    }
    let mut seen = std::collections::HashSet::new();
    for rule in rules {
        if rule.id.trim().is_empty() || !seen.insert(rule.id.as_str()) {
            out.push(Problem::new("id", format!("Identifiant de règle vide ou en double : « {} ».", rule.id)));
        }
    }
    out
}
