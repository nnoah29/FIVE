//! État de l'interface : emplacement, éléments, sélection, filtres, presse-papiers.

use std::collections::{BTreeMap, HashSet};

use five_engine::entry::Entry;
use serde_json::Value;

use crate::model::sorting::SortKey;

/// Emplacement de la barre latérale (dossiers XDG, corbeille).
#[derive(Debug, Clone)]
pub struct Place {
    pub id: String,
    pub label: String,
    pub path: String,
}

/// Disque monté affiché sous les emplacements.
#[derive(Debug, Clone)]
pub struct Disk {
    pub name: String,
    pub mount: String,
    pub fs: String,
    pub total: u64,
    pub free: u64,
    pub path: Option<String>,
}

/// Éléments copiés ou coupés en attente de collage.
#[derive(Debug, Clone)]
pub struct Clip {
    pub mode: String,
    pub paths: Vec<String>,
}

/// Analyse d'occupation du dossier courant.
#[derive(Debug, Clone)]
pub struct UsageView {
    pub path: String,
    pub data: Option<Value>,
}

/// Tout ce que l'interface sait de la situation courante.
pub struct State {
    pub path: String,
    pub items: Vec<Entry>,
    pub shown: Vec<Entry>,
    pub sel: isize,
    pub picked: HashSet<String>,
    pub anchor: isize,
    pub clip: Option<Clip>,
    pub usage: Option<UsageView>,
    pub git: BTreeMap<String, char>,
    pub git_path: Option<String>,
    pub pending: Option<String>,
    pub sort: SortKey,
    pub ascending: bool,
    pub filter: String,
    pub hidden: bool,
    pub settings_open: bool,
    pub tab: String,
    pub virtual_view: Option<String>,
    pub query: String,
    pub places: Vec<Place>,
    pub disks: Vec<Disk>,
    pub root: String,
    pub user: String,
    pub free: u64,
    pub total: u64,
    pub ticket: u64,
}

impl State {
    /// État initial avant la première lecture.
    pub fn new(root: String, user: String) -> State {
        State {
            path: String::new(),
            items: Vec::new(),
            shown: Vec::new(),
            sel: -1,
            picked: HashSet::new(),
            anchor: -1,
            clip: None,
            usage: None,
            git: BTreeMap::new(),
            git_path: None,
            pending: None,
            sort: SortKey::Name,
            ascending: true,
            filter: "all".into(),
            hidden: false,
            settings_open: false,
            tab: "display".into(),
            virtual_view: None,
            query: String::new(),
            places: Vec::new(),
            disks: Vec::new(),
            root,
            user,
            free: 0,
            total: 0,
            ticket: 0,
        }
    }

    /// Une recherche est en cours (hors vues virtuelles).
    pub fn searching(&self) -> bool {
        !self.query.trim().is_empty() && self.virtual_view.is_none()
    }

    /// La liste montre aussi les dossiers (recherche ou vue virtuelle).
    pub fn list_all(&self) -> bool {
        self.searching() || self.virtual_view.is_some()
    }

    /// On peut créer ou coller ici : dossier ordinaire, hors recherche et hors corbeille.
    pub fn can_create(&self) -> bool {
        self.virtual_view.is_none() && !self.searching() && !crate::model::paths::in_trash(&self.path)
    }

    /// Éléments sélectionnés, dans l'ordre d'affichage.
    pub fn picked_items(&self) -> Vec<Entry> {
        self.shown.iter().filter(|item| self.picked.contains(&item.path)).cloned().collect()
    }
}
