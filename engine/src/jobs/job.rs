//! Un transfert en cours : progression partagée entre le thread de copie et l'API.

use std::path::PathBuf;
use std::sync::{Condvar, Mutex};

use serde_json::{Value, json};

use super::state::{Kind, State, Stop};
use crate::ids::unique_id;
use crate::sync::lock;

/// Progression modifiable d'un transfert.
#[derive(Debug)]
pub struct Progress {
    pub state: State,
    pub error: String,
    pub total: u64,
    pub done: u64,
    pub files: u64,
    pub files_done: u64,
    pub current: String,
}

/// Transfert de fichiers : copie ou déplacement.
#[derive(Debug)]
pub struct Job {
    pub id: String,
    pub kind: Kind,
    pub sources: Vec<PathBuf>,
    pub dest: PathBuf,
    pub dest_rel: String,
    progress: Mutex<Progress>,
    gate: Condvar,
}

impl Job {
    /// Crée un transfert en cours, sans progression.
    pub fn new(kind: Kind, sources: Vec<PathBuf>, dest: PathBuf, dest_rel: String) -> Job {
        let progress = Progress { state: State::Running, error: String::new(), total: 0, done: 0, files: 0, files_done: 0, current: String::new() };
        Job { id: unique_id()[..10].to_string(), kind, sources, dest, dest_rel, progress: Mutex::new(progress), gate: Condvar::new() }
    }

    /// Modifie la progression et réveille le thread en pause.
    pub fn update(&self, change: impl FnOnce(&mut Progress)) {
        change(&mut lock(&self.progress));
        self.gate.notify_all();
    }

    /// État courant.
    pub fn state(&self) -> State {
        lock(&self.progress).state
    }

    /// Point d'arrêt : attend si le transfert est en pause, échoue s'il est annulé.
    pub fn checkpoint(&self) -> Result<(), Stop> {
        let mut progress = lock(&self.progress);
        while progress.state == State::Paused {
            progress = self.gate.wait(progress).unwrap_or_else(|e| e.into_inner());
        }
        if progress.state == State::Cancelled { Err(Stop::Cancelled) } else { Ok(()) }
    }

    /// Représentation JSON attendue par l'interface.
    pub fn view(&self) -> Value {
        let p = lock(&self.progress);
        json!({"id": self.id, "kind": self.kind.as_str(), "state": p.state.as_str(), "error": p.error,
            "total": p.total, "done": p.done, "files": p.files, "filesDone": p.files_done,
            "current": p.current, "dest": self.dest_rel, "count": self.sources.len()})
    }
}
