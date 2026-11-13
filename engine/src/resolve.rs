//! Résolution confinée des chemins relatifs envoyés par l'interface.

use std::fs;
use std::path::PathBuf;

use crate::realpath::real_path;
use crate::refus::{Refus, Resultat};
use crate::root::Root;

impl Root {
    /// Résout un chemin relatif en chemin absolu confiné à la racine.
    pub fn resolve(&self, rel: &str) -> Resultat<PathBuf> {
        let full = real_path(&self.path().join(rel.trim_matches('/')));
        if !self.inside(&full) {
            return Err(Refus::new(403, "Chemin hors de la racine autorisée"));
        }
        Ok(full)
    }

    /// Résout un élément existant sans suivre son dernier lien symbolique.
    pub fn resolve_entry(&self, rel: &str) -> Resultat<PathBuf> {
        let (parent, name) = rel.trim_matches('/').rsplit_once('/').unwrap_or(("", rel.trim_matches('/')));
        if name.is_empty() || name == "." || name == ".." {
            return Err(Refus::new(400, "Chemin invalide"));
        }
        let full = self.resolve(parent)?.join(name);
        if full == self.path() || fs::symlink_metadata(&full).is_err() {
            return Err(Refus::new(404, "Élément introuvable"));
        }
        Ok(full)
    }
}
