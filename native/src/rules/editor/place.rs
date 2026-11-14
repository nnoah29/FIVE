//! Où se trouve un élément de l'éditeur : liste du catalogue, tableau qui le contient, rang, chemins dans le brouillon.

use crate::model::rules_draft::{Seg, key};
use crate::model::rules_labels::{action_label, condition_label, trigger_label};

/// Où se trouve l'élément : liste du catalogue (`triggers`, `conditions`, `actions`), tableau qui le contient, rang.
pub struct Place {
    pub list: &'static str,
    pub container: Vec<Seg>,
    pub index: usize,
    pub total: usize,
}

/// Libellé français d'un type selon la liste.
pub fn kind_label(list: &str, kind: &str) -> String {
    match list {
        "triggers" => trigger_label(kind),
        "conditions" => condition_label(kind),
        _ => action_label(kind),
    }
}

impl Place {
    /// Chemin de l'élément dans le brouillon.
    pub fn item(&self) -> Vec<Seg> {
        [self.container.clone(), vec![Seg::Idx(self.index)]].concat()
    }

    /// Chemin de l'objet qui porte le type et les champs (pour une condition : sa clé `test`).
    pub fn body(&self) -> Vec<Seg> {
        if self.list == "conditions" { [self.item(), vec![key("test")]].concat() } else { self.item() }
    }
}
