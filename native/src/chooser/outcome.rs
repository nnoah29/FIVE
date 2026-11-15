//! Résultat d'une sélection renvoyé au portail : adresses `file://` et filtre choisi.

use std::path::PathBuf;

use gtk4::gio;
use gtk4::glib::{Variant, VariantDict};
use gtk4::prelude::*;

use super::request::Request;

/// Ce que l'utilisateur a choisi : des chemins et le rang du filtre actif.
#[derive(Debug, Clone, PartialEq)]
pub struct Choice {
    pub paths: Vec<PathBuf>,
    pub filter: Option<usize>,
}

/// Dictionnaire `a{sv}` du résultat : `uris` et, s'il y a des filtres, `current_filter`.
pub fn results_variant(request: &Request, choice: &Choice) -> Variant {
    let uris: Vec<String> = choice.paths.iter().map(|p| gio::File::for_path(p).uri().to_string()).collect();
    let dict = VariantDict::new(None);
    dict.insert_value("uris", &uris.to_variant());
    if let Some(filter) = choice.filter.and_then(|i| request.filters.get(i)) {
        let rules: Vec<(u32, String)> = filter.rules.iter().map(|r| (r.kind, r.pattern.clone())).collect();
        dict.insert_value("current_filter", &(filter.name.clone(), rules).to_variant());
    }
    dict.end()
}
