//! Validation des noms de fichiers saisis par l'utilisateur.

use crate::refus::{Refus, Resultat};

/// Retourne le nom nettoyé, ou refuse un nom vide, relatif ou trop long.
pub fn clean_name(name: &str) -> Resultat<String> {
    let name = name.trim();
    let invalid = name.is_empty()
        || name == "."
        || name == ".."
        || name.contains('/')
        || name.contains('\0')
        || name.len() > 255;
    if invalid {
        return Err(Refus::new(400, "Nom invalide : sans « / », ni vide, 255 octets au plus"));
    }
    Ok(name.to_string())
}
