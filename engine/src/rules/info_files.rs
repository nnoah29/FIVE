//! Informations lues dans le fichier : texte, somme SHA-256, archive, origine du téléchargement, état Git.

use std::fs::File;
use std::io::Read;
use std::path::Path;

use sha2::{Digest, Sha256};

use super::context::ArchiveInfo;
use crate::archive::list_archive;
use crate::git_marks::git_marks;
use crate::text::read_text;

/// Texte (200 Ko au plus) et encodage : `utf8` si le fichier est du texte, `other` sinon.
pub fn text_of(path: &Path) -> (Option<String>, Option<String>) {
    let Ok(data) = read_text(path) else { return (None, None) };
    if data["binary"].as_bool().unwrap_or(false) {
        return (None, Some("other".into()));
    }
    (data["text"].as_str().map(String::from), Some("utf8".into()))
}

/// Somme SHA-256 du fichier, en hexadécimal.
pub fn hash_of(path: &Path) -> Option<String> {
    let mut file = File::open(path).ok()?;
    let (mut hasher, mut block) = (Sha256::new(), vec![0u8; 1 << 16]);
    loop {
        let read = file.read(&mut block).ok()?;
        if read == 0 {
            return Some(hasher.finalize().iter().map(|b| format!("{b:02x}")).collect());
        }
        hasher.update(&block[..read]);
    }
}

/// Contenu d'une archive : lisible, noms des entrées (500 au plus).
pub fn archive_of(path: &Path) -> ArchiveInfo {
    match list_archive(path) {
        Ok(data) => ArchiveInfo {
            readable: true,
            encrypted: false,
            entries: data["items"].as_array().map(|a| a.iter().filter_map(|i| i["n"].as_str().map(String::from)).collect()).unwrap_or_default(),
        },
        Err(_) => ArchiveInfo::default(),
    }
}

/// État Git du fichier : `modified`, `added`, `deleted`, `untracked`, `ignored`, `conflict`.
pub fn git_status_of(path: &Path) -> Option<String> {
    let (parent, name) = (path.parent()?, path.file_name()?.to_string_lossy().into_owned());
    let code = git_marks(parent).get(&name).copied()?;
    Some(
        match code {
            'M' => "modified",
            'A' => "added",
            'D' => "deleted",
            '?' => "untracked",
            '!' => "ignored",
            _ => "conflict",
        }
        .to_string(),
    )
}

/// Adresse d'origine du téléchargement (attribut étendu `user.xdg.origin.url`), si le système la fournit.
pub fn origin_of(path: &Path) -> Option<String> {
    let mut buffer = [0u8; 2048];
    let length = rustix::fs::getxattr(path, "user.xdg.origin.url", &mut buffer).ok()?;
    Some(String::from_utf8_lossy(&buffer[..length]).into_owned())
}
