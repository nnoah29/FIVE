//! Aperçus consultés : une règle ne sort de l'essai que si son aperçu actuel a été vu.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use crate::refus::Resultat;

/// Aperçus consultés : identifiant de règle vers empreinte de la règle prévisualisée.
pub fn load_previews(dir: &Path) -> BTreeMap<String, String> {
    fs::read_to_string(dir.join("apercus-regles.json")).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default()
}

/// Enregistre les aperçus consultés.
pub fn save_previews(dir: &Path, previews: &BTreeMap<String, String>) -> Resultat<()> {
    fs::create_dir_all(dir)?;
    Ok(fs::write(dir.join("apercus-regles.json"), serde_json::to_vec(previews).unwrap_or_default())?)
}
