//! Identifiant d'une passe de règles : regroupe ses lignes de journal pour l'annulation.

use crate::ids::unique_id;

/// Nouvel identifiant de passe (8 caractères).
pub fn new_run_id() -> String {
    unique_id().chars().take(8).collect()
}
