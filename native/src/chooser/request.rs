//! Demande de sélection : mode, titre, filtres, nom et dossier de départ proposés.

use std::path::PathBuf;

/// Ce que l'application demande : ouvrir des fichiers, choisir des dossiers ou enregistrer un fichier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Open,
    Folder,
    Save,
}

/// Une règle d'un filtre : motif de nom (`kind` 0) ou type MIME (`kind` 1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilterRule {
    pub kind: u32,
    pub pattern: String,
}

/// Un filtre nommé, par exemple « Images ».
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Filter {
    pub name: String,
    pub rules: Vec<FilterRule>,
}

/// Demande complète transmise par l'application ou le portail.
#[derive(Debug, Clone, PartialEq)]
pub struct Request {
    pub mode: Mode,
    pub title: String,
    pub accept_label: Option<String>,
    pub multiple: bool,
    pub filters: Vec<Filter>,
    pub current_filter: usize,
    pub current_name: String,
    pub start_dir: PathBuf,
}

impl Request {
    /// Demande par défaut : un seul élément, aucun filtre, départ dans le dossier personnel.
    pub fn new(mode: Mode, title: &str) -> Request {
        let home = std::env::var_os("HOME").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("/"));
        Request {
            mode,
            title: title.to_string(),
            accept_label: None,
            multiple: false,
            filters: Vec::new(),
            current_filter: 0,
            current_name: String::new(),
            start_dir: home,
        }
    }
}
