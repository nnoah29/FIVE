//! Lecture des options du portail (`a{sv}`) : filtres, nom proposé, dossier de départ.

use std::path::PathBuf;

use gtk4::glib::{Variant, VariantDict};

pub use super::bytes::{bytes_of, path_from_bytes};
use super::request::{Filter, FilterRule, Mode, Request};

type RawFilter = (String, Vec<(u32, String)>);

/// Convertit un filtre brut `(nom, [(type, motif)])` en filtre.
fn to_filter((name, rules): RawFilter) -> Filter {
    Filter { name, rules: rules.into_iter().map(|(kind, pattern)| FilterRule { kind, pattern }).collect() }
}

/// Lit les filtres proposés et retourne leur liste avec le rang du filtre courant.
fn read_filters(dict: &VariantDict) -> (Vec<Filter>, usize) {
    let all: Vec<RawFilter> = dict.lookup_value("filters", None).and_then(|v| v.get()).unwrap_or_default();
    let filters: Vec<Filter> = all.into_iter().map(to_filter).collect();
    let current: Option<RawFilter> = dict.lookup_value("current_filter", None).and_then(|v| v.get());
    let rank = current.and_then(|c| filters.iter().position(|f| f.name == c.0)).unwrap_or(0);
    (filters, rank)
}

/// Dossier de départ : celui qui est demandé, sinon le dossier du fichier proposé.
fn start_dir(dict: &VariantDict) -> Option<PathBuf> {
    let bytes = |key: &str| dict.lookup_value(key, None).and_then(|v| v.get::<Vec<u8>>()).map(|b| path_from_bytes(&b));
    bytes("current_folder").or_else(|| bytes("current_file").and_then(|f| f.parent().map(PathBuf::from)))
}

/// Construit la demande depuis le mode, le titre et le dictionnaire d'options du portail.
pub fn parse_request(mode: Mode, title: &str, options: &Variant) -> Request {
    let dict = VariantDict::new(Some(options));
    let mut request = Request::new(mode, title);
    request.multiple = dict.lookup::<bool>("multiple").ok().flatten().unwrap_or(false);
    request.accept_label = dict.lookup::<String>("accept_label").ok().flatten().filter(|l| !l.is_empty());
    request.current_name = dict.lookup::<String>("current_name").ok().flatten().unwrap_or_default();
    (request.filters, request.current_filter) = read_filters(&dict);
    if let Some(dir) = start_dir(&dict).filter(|d| d.is_dir()) {
        request.start_dir = dir;
    }
    request
}

/// Mode d'une demande d'ouverture : « Dossier » si l'option `directory` est vraie, sinon « Ouvrir ».
pub fn open_mode(options: &Variant) -> Mode {
    let dict = VariantDict::new(Some(options));
    if dict.lookup::<bool>("directory").ok().flatten().unwrap_or(false) { Mode::Folder } else { Mode::Open }
}
