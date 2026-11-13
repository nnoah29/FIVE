//! Création de dossiers.

use std::fs;

use crate::names::clean_name;
use crate::refus::{Refus, Resultat};
use crate::root::Root;

/// Crée un dossier `name` dans `parent_rel` et retourne son chemin relatif.
pub fn make_folder(root: &Root, parent_rel: &str, name: &str) -> Resultat<String> {
    let parent = root.resolve(parent_rel)?;
    if root.in_trash(&parent.join("x")) {
        return Err(Refus::new(400, "Impossible de créer un dossier dans la corbeille"));
    }
    let target = parent.join(clean_name(name)?);
    fs::create_dir(&target)?;
    Ok(root.relative(&target))
}
