//! Changements entre deux balayages : nouveau fichier, fichier modifié, fichier qui ne bouge plus.

use std::collections::HashSet;

use super::scan::Seen;
use super::state::{RulesState, Tracked};

/// Durée minimale sans changement avant de signaler un fichier stable.
const MIN_STABLE: f64 = 2.0;

/// Nature d'un changement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeKind {
    Created,
    Modified,
    Stable,
}

/// Un changement détecté, avec depuis combien de temps le fichier ne bouge plus.
#[derive(Debug, Clone, PartialEq)]
pub struct Change {
    pub path: String,
    pub kind: ChangeKind,
    pub stable_for: f64,
    pub is_dir: bool,
}

/// Compare un élément vu à ce qu'on en savait, met à jour la mémoire, et retourne le changement éventuel.
fn step(state: &mut RulesState, seen: &Seen, now: f64, max_wait: f64) -> Option<Change> {
    let change = |kind, stable_for| Some(Change { path: seen.path.clone(), kind, stable_for, is_dir: seen.is_dir });
    let fresh = Tracked { size: seen.size, mtime: seen.mtime, since: now, done: false };
    let Some(known) = state.tracked.get_mut(&seen.path) else {
        state.tracked.insert(seen.path.clone(), fresh);
        return change(ChangeKind::Created, 0.0);
    };
    if known.size != seen.size || known.mtime != seen.mtime {
        *known = fresh;
        return change(ChangeKind::Modified, 0.0);
    }
    let (still, was_done) = (now - known.since, known.done);
    known.done |= still >= max_wait;
    if was_done || still < MIN_STABLE {
        return None;
    }
    change(ChangeKind::Stable, still)
}

/// Détecte les changements d'une passe complète ; la première passe ne fait que mémoriser (les fichiers déjà là ne sont pas retraités).
pub fn detect(state: &mut RulesState, seen: &[Seen], now: f64, max_wait: f64, complete: bool) -> Vec<Change> {
    let baseline = !state.baselined;
    let changes: Vec<Change> = seen.iter().filter_map(|s| step(state, s, now, max_wait)).collect();
    if complete {
        let present: HashSet<&str> = seen.iter().map(|s| s.path.as_str()).collect();
        state.tracked.retain(|path, _| present.contains(path.as_str()));
        state.baselined = true;
    }
    if baseline { Vec::new() } else { changes }
}
