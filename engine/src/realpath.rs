//! Résolution de chemin façon `realpath`, tolérante aux chemins qui n'existent pas encore.

use std::fs;
use std::path::{Component, Path, PathBuf};

/// Résout les liens symboliques et les `..` d'un chemin, même si la fin n'existe pas.
pub fn real_path(path: &Path) -> PathBuf {
    let mut resolved = PathBuf::new();
    for part in path.components() {
        match part {
            Component::RootDir | Component::Prefix(_) => resolved = PathBuf::from("/"),
            Component::CurDir => {}
            Component::ParentDir => {
                resolved.pop();
            }
            Component::Normal(name) => {
                resolved.push(name);
                follow_link(&mut resolved);
            }
        }
    }
    resolved
}

/// Remplace le chemin par sa cible si c'est un lien symbolique qui existe.
fn follow_link(resolved: &mut PathBuf) {
    let is_link = fs::symlink_metadata(&*resolved).is_ok_and(|info| info.file_type().is_symlink());
    if !is_link {
        return;
    }
    if let Ok(target) = fs::canonicalize(&*resolved) {
        *resolved = target;
    }
}
