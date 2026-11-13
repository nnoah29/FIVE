//! Mise à la corbeille XDG et effacement définitif.

use std::fs::{self, DirBuilder};
use std::os::unix::fs::DirBuilderExt;
use std::path::Path;

use crate::clock::local_stamp;
use crate::refus::{Refus, Resultat};
use crate::root::Root;
use crate::urlcodec::encode_path;

/// Premier nom libre dans `files` : `name`, puis `name.2`, `name.3`…
pub fn unique_trash_name(files: &Path, name: &str) -> String {
    let mut candidate = name.to_string();
    let mut n = 1;
    while fs::symlink_metadata(files.join(&candidate)).is_ok() {
        n += 1;
        candidate = format!("{name}.{n}");
    }
    candidate
}

/// Déplace un élément dans la corbeille et écrit son `.trashinfo`.
pub fn move_to_trash(root: &Root, full: &Path) -> Resultat<()> {
    let base = root.trash_dir();
    let (files, infos) = (base.join("files"), base.join("info"));
    for folder in [&files, &infos] {
        DirBuilder::new().recursive(true).mode(0o700).create(folder)?;
    }
    let leaf = full.file_name().map_or_else(String::new, |n| n.to_string_lossy().into_owned());
    let name = unique_trash_name(&files, &leaf);
    let info = infos.join(format!("{name}.trashinfo"));
    let text = format!("[Trash Info]\nPath={}\nDeletionDate={}\n", encode_path(&full.to_string_lossy()), local_stamp());
    fs::write(&info, text)?;
    if fs::rename(full, files.join(&name)).is_err() {
        let _ = fs::remove_file(&info);
        return Err(Refus::new(409, "Cet élément est sur un autre disque que la corbeille : suppression refusée"));
    }
    Ok(())
}

/// Efface définitivement un élément de la corbeille, avec son `.trashinfo`.
pub fn erase_for_good(root: &Root, full: &Path) -> Resultat<()> {
    if fs::symlink_metadata(full)?.is_dir() {
        fs::remove_dir_all(full)?;
    } else {
        fs::remove_file(full)?;
    }
    let name = full.file_name().map_or_else(String::new, |n| n.to_string_lossy().into_owned());
    if full.parent() == Some(root.trash_files().as_path()) {
        let _ = fs::remove_file(root.trash_dir().join("info").join(format!("{name}.trashinfo")));
    }
    Ok(())
}
