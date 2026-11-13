//! Démarrage et contrôle des transferts.

use std::sync::Arc;
use std::thread;

use super::job::Job;
use super::measure::measure;
use super::registry::Jobs;
use super::state::{Kind, State};
use super::transfer::run_job;
use crate::refus::{Refus, Resultat};
use crate::root::Root;

/// Vérifie les sources et la destination, puis lance le transfert ; retourne son identifiant.
pub fn start_job(root: &Root, jobs: &Jobs, kind: &str, rels: &[String], dest_rel: &str) -> Resultat<String> {
    let kind = Kind::parse(kind).ok_or_else(|| Refus::new(400, "Type de transfert inconnu"))?;
    let dest = root.resolve(dest_rel)?;
    if !dest.is_dir() || root.in_trash(&dest.join("x")) {
        return Err(Refus::new(400, "Destination invalide"));
    }
    let sources = rels.iter().map(|rel| root.resolve_entry(rel)).collect::<Resultat<Vec<_>>>()?;
    for src in &sources {
        if src.is_dir() && !src.is_symlink() && dest.starts_with(src) {
            return Err(Refus::new(400, "Impossible de copier un dossier dans lui-même"));
        }
        if kind == Kind::Move && src.parent() == Some(dest.as_path()) {
            return Err(Refus::new(400, "Les éléments sont déjà dans ce dossier"));
        }
    }
    let job = Arc::new(Job::new(kind, sources, dest, root.relative(&root.resolve(dest_rel)?)));
    let measured: Vec<(u64, u64)> = job.sources.iter().map(|s| measure(s)).collect();
    job.update(|p| measured.iter().for_each(|(size, count)| (p.total, p.files) = (p.total + size, p.files + count)));
    jobs.insert(job.clone());
    let id = job.id.clone();
    thread::spawn(move || run_job(job));
    Ok(id)
}

/// Met en pause, reprend ou annule un transfert.
pub fn control_job(jobs: &Jobs, id: &str, action: &str) -> Resultat<()> {
    let job = jobs.find(id).ok_or_else(|| Refus::new(404, "Transfert introuvable"))?;
    let next = match (action, job.state()) {
        ("pause", State::Running) => Some(State::Paused),
        ("resume", State::Paused) => Some(State::Running),
        ("cancel", live) if live.is_live() => Some(State::Cancelled),
        ("pause" | "resume" | "cancel", _) => None,
        _ => return Err(Refus::new(400, "Action inconnue")),
    };
    if let Some(state) = next {
        job.update(|p| if p.state.is_live() { p.state = state });
    }
    Ok(())
}
