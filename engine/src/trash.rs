//! Emplacement de la corbeille XDG et reconnaissance de son contenu.

use std::path::{Path, PathBuf};

use crate::realpath::real_path;
use crate::root::Root;

impl Root {
    /// Dossier de la corbeille XDG de l'utilisateur.
    pub fn trash_dir(&self) -> PathBuf {
        self.home().join(".local/share/Trash")
    }

    /// Dossier des fichiers de la corbeille, liens résolus.
    pub fn trash_files(&self) -> PathBuf {
        real_path(&self.trash_dir()).join("files")
    }

    /// Indique si le chemin est dans le dossier des fichiers de la corbeille.
    pub fn in_trash(&self, full: &Path) -> bool {
        let base = self.trash_files();
        full.starts_with(&base) && full != base
    }

    /// Indique si le chemin contient la corbeille ou en fait partie structurelle.
    pub fn protects_trash(&self, full: &Path) -> bool {
        let base = real_path(&self.trash_dir());
        full == base || base.starts_with(full) || full == self.trash_files()
    }
}
