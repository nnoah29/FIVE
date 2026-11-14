//! Chemins d'extraction : un nom d'entrée ne peut jamais sortir du dossier de destination.

use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};

/// Chemin de l'entrée sous `folder` ; `None` pour un nom vide, erreur pour un nom qui remonte (`..`) ou contient un octet nul.
pub fn safe_join(folder: &Path, name: &str) -> io::Result<Option<PathBuf>> {
    if name.contains('\0') {
        return Err(ErrorKind::InvalidData.into());
    }
    let mut target = folder.to_path_buf();
    let mut parts = 0;
    for part in name.split('/').filter(|part| !part.is_empty() && *part != ".") {
        if part == ".." {
            return Err(ErrorKind::InvalidData.into());
        }
        target.push(part);
        parts += 1;
    }
    Ok((parts > 0).then_some(target))
}
