//! API de lecture des règles : liste avec statuts, catalogue, journal.

use std::path::Path;

use serde_json::{Value, json};

use super::api_support::{rules_file, view};
use super::catalog::catalog;
use super::journal::{journal_path, read_recent};
use super::store::load;
use crate::root::Root;

/// Liste des règles dans l'ordre d'évaluation, chacune avec son statut ; signale entrées illisibles et fichier abîmé.
pub fn list_rules(root: &Root, dir: &Path) -> Value {
    let stored = load(&rules_file(dir));
    let rules: Vec<Value> = stored.rules.iter().map(|r| view(root, r)).collect();
    json!({"rules": rules, "unreadable": stored.unreadable.len(), "damaged": stored.damaged, "newer": stored.newer})
}

/// Catalogue des conditions, actions, déclencheurs et valeurs permises.
pub fn rules_catalog() -> Value {
    catalog()
}

/// Les `limit` dernières lignes du journal (1 à 500), de la plus récente à la plus ancienne.
pub fn rules_history(dir: &Path, limit: usize) -> Value {
    json!({"items": read_recent(&journal_path(dir), limit.clamp(1, 500))})
}
