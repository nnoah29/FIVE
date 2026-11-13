//! La racine exposée : tout chemin du moteur y est confiné.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Racine exposée et dossier personnel (qui porte la corbeille).
#[derive(Debug, Clone)]
pub struct Root {
    path: PathBuf,
    home: PathBuf,
}

impl Root {
    /// Ouvre la racine donnée ; le dossier doit exister.
    pub fn new(path: &Path, home: &Path) -> io::Result<Self> {
        Ok(Self { path: fs::canonicalize(path)?, home: home.to_path_buf() })
    }

    /// Chemin absolu de la racine.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Dossier personnel de l'utilisateur.
    pub fn home(&self) -> &Path {
        &self.home
    }

    /// Indique si un chemin absolu est dans la racine (ou est la racine).
    pub fn inside(&self, full: &Path) -> bool {
        full.starts_with(&self.path)
    }

    /// Chemin relatif à la racine, vide pour la racine elle-même.
    pub fn relative(&self, full: &Path) -> String {
        let rel = full.strip_prefix(&self.path).unwrap_or(full);
        rel.to_string_lossy().into_owned()
    }
}
