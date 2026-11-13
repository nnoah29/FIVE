//! Suppression : corbeille d'abord, effacement définitif seulement depuis la corbeille.

use crate::refus::{Refus, Resultat};
use crate::root::Root;
use crate::trash_ops::{erase_for_good, move_to_trash};

/// Façon dont un élément a été supprimé.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Deleted {
    Trash,
    Definitive,
}

impl Deleted {
    /// Nom du mode tel que l'API le renvoie.
    pub fn as_str(self) -> &'static str {
        match self {
            Deleted::Trash => "trash",
            Deleted::Definitive => "definitive",
        }
    }
}

/// Supprime un élément ; refuse la racine et tout dossier qui contient la corbeille.
pub fn delete_entry(root: &Root, rel: &str) -> Resultat<Deleted> {
    let full = root.resolve_entry(rel)?;
    if root.protects_trash(&full) {
        return Err(Refus::new(400, "Ce dossier est protégé : il contient la corbeille"));
    }
    if root.in_trash(&full) {
        erase_for_good(root, &full)?;
        return Ok(Deleted::Definitive);
    }
    move_to_trash(root, &full)?;
    Ok(Deleted::Trash)
}
