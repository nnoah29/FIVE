//! Aides de l'exécution : nom d'une action, attente d'un transfert, chemins sûrs, destination libre.

use std::fs;
use std::path::{Path, PathBuf};
use std::thread::sleep;
use std::time::Duration;

use super::run_types::{Fail, RunEnv};
use super::types_action::RuleAction;
use super::types_rule::ConflictPolicy;
use crate::jobs::copy::unique_destination;
use crate::jobs::start::start_job;

/// Nom de l'action tel qu'il est écrit dans le journal (`move_to`, `rename`…).
pub fn action_name(action: &RuleAction) -> String {
    serde_json::to_value(action).ok().and_then(|v| v["type"].as_str().map(String::from)).unwrap_or_default()
}

/// Chemin relatif sans `..`, sans chemin absolu ni octet nul : jamais de sortie du dossier voulu.
pub fn safe_relative(text: &str) -> Result<String, Fail> {
    let parts: Vec<&str> = text.split('/').filter(|p| !p.is_empty() && *p != ".").collect();
    if parts.contains(&"..") || text.contains('\0') {
        return Err(Fail::Error("Chemin refusé : il remonte hors du dossier".into()));
    }
    Ok(parts.join("/"))
}

/// Attend la fin d'un transfert (vérifie toutes les 20 ms) ; annulé ou en erreur, il devient un échec.
pub fn wait_job(env: &RunEnv, id: &str) -> Result<(), Fail> {
    loop {
        let Some(job) = env.jobs.find(id) else { return Err(Fail::Error("Transfert introuvable".into())) };
        let view = job.view();
        match view["state"].as_str() {
            Some("running" | "paused") => sleep(Duration::from_millis(20)),
            Some("done") => return Ok(()),
            Some("cancelled") => return Err(Fail::Skip("transfert annulé".into())),
            _ => return Err(Fail::Error(view["error"].as_str().unwrap_or("Erreur système").to_string())),
        }
    }
}

/// Destination libre pour `name` dans `folder` selon la politique : un nom `nom (2)`, un refus, ou une question.
pub fn target_free(folder: &Path, name: &str, policy: ConflictPolicy) -> Result<PathBuf, Fail> {
    if fs::symlink_metadata(folder.join(name)).is_err() {
        return Ok(folder.join(name));
    }
    match policy {
        ConflictPolicy::Rename => Ok(unique_destination(folder, name)),
        ConflictPolicy::Skip => Err(Fail::Skip(format!("« {name} » existe déjà à destination"))),
        ConflictPolicy::Ask => Err(Fail::Ask),
    }
}

/// Dernier composant d'un chemin.
pub fn leaf(path: &Path) -> String {
    path.file_name().map_or_else(String::new, |n| n.to_string_lossy().into_owned())
}

/// Lance un transfert et attend sa fin.
pub fn run_job(env: &RunEnv, kind: &str, paths: &[String], dest: &str) -> Result<(), Fail> {
    let id = start_job(env.root, env.jobs, kind, paths, dest)?;
    wait_job(env, &id)
}
