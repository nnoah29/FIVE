//! Registre des transferts connus : les plus récents terminés sont conservés.

use std::sync::{Arc, Mutex};

use serde_json::Value;

use super::job::Job;
use crate::sync::lock;

const FINISHED_KEPT: usize = 12;

/// Liste partagée des transferts, du plus ancien au plus récent.
#[derive(Debug, Default)]
pub struct Jobs {
    list: Mutex<Vec<Arc<Job>>>,
}

impl Jobs {
    /// Registre vide.
    pub fn new() -> Jobs {
        Jobs::default()
    }

    /// Ajoute un transfert et oublie les plus anciens transferts terminés.
    pub fn insert(&self, job: Arc<Job>) {
        let mut list = lock(&self.list);
        let ended: Vec<String> = list.iter().filter(|j| !j.state().is_live()).map(|j| j.id.clone()).collect();
        let forgotten = ended.len().saturating_sub(FINISHED_KEPT);
        list.retain(|j| !ended[..forgotten].contains(&j.id));
        list.push(job);
    }

    /// Retrouve un transfert par son identifiant.
    pub fn find(&self, id: &str) -> Option<Arc<Job>> {
        lock(&self.list).iter().find(|j| j.id == id).cloned()
    }

    /// Vue JSON de tous les transferts.
    pub fn view(&self) -> Vec<Value> {
        lock(&self.list).iter().map(|j| j.view()).collect()
    }
}
