//! Types de l'exécution : résultat d'une action, annulation, échec, environnement.

use std::io;

use serde::{Deserialize, Serialize};

use crate::jobs::registry::Jobs;
use crate::refus::Refus;
use crate::root::Root;

/// Comment défaire une action ; les chemins sont absolus.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Undo {
    Move { from: String, to: String },
    Trashed { original: String, trashed: String },
    TrashCreated { path: String },
    RemoveEmptyDir { path: String },
    Recreate { path: String },
    Mode { path: String, mode: u32 },
    Time { path: String, seconds: f64 },
}

/// Issue d'une action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    Done,
    DryRun,
    Skipped,
    Failed,
    Deferred,
    Ask,
}

/// Résultat d'une action, tel qu'il est journalisé.
#[derive(Debug, Clone, PartialEq)]
pub struct ActionResult {
    pub rule_id: String,
    pub path: String,
    pub action: String,
    pub outcome: Outcome,
    pub detail: String,
    pub undo: Vec<Undo>,
}

/// Action réussie : ce qui s'est passé et comment le défaire.
#[derive(Debug, Clone, Default)]
pub struct Done {
    pub detail: String,
    pub undo: Vec<Undo>,
}

/// Raison pour laquelle une action n'a pas abouti.
#[derive(Debug, Clone, PartialEq)]
pub enum Fail {
    Skip(String),
    Error(String),
    Ask,
    Defer,
}

/// Résultat d'une étape d'exécution.
pub type Step = Result<Done, Fail>;

impl From<Refus> for Fail {
    /// Un refus du moteur devient un échec lisible.
    fn from(refus: Refus) -> Self {
        Fail::Error(refus.message)
    }
}

impl From<io::Error> for Fail {
    /// Une erreur système devient un échec lisible.
    fn from(error: io::Error) -> Self {
        Fail::Error(Refus::from(error).message)
    }
}

/// Ce que l'exécution utilise : la racine et la file de transferts.
pub struct RunEnv<'a> {
    pub root: &'a Root,
    pub jobs: &'a Jobs,
}
