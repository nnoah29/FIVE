//! Stockage des règles dans `~/.config/five/regles.json` : écriture atomique, fichier abîmé mis de côté.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde_json::{Value, json};

use super::types_rule::{Rule, SCHEMA_VERSION};
use crate::refus::Resultat;

/// Règles lues : celles qu'on comprend, celles qu'on conserve telles quelles, et les anomalies.
#[derive(Debug, Clone, Default)]
pub struct Stored {
    pub rules: Vec<Rule>,
    pub unreadable: Vec<Value>,
    pub damaged: bool,
    pub newer: bool,
}

/// Emplacement du fichier de règles sous le dossier personnel.
pub fn rules_path(home: &Path) -> PathBuf {
    home.join(".config/five/regles.json")
}

/// Sépare les entrées lisibles des autres, qui sont conservées sans modification.
fn parse_rules(items: &[Value]) -> (Vec<Rule>, Vec<Value>) {
    let (mut rules, mut unreadable) = (Vec::new(), Vec::new());
    for item in items {
        match serde_json::from_value::<Rule>(item.clone()) {
            Ok(rule) => rules.push(rule),
            Err(_) => unreadable.push(item.clone()),
        }
    }
    (rules, unreadable)
}

/// Lit les règles ; un fichier absent donne une liste vide, un fichier abîmé est renommé `.abime`, une version plus récente est conservée intacte.
pub fn load(path: &Path) -> Stored {
    let Ok(text) = fs::read_to_string(path) else { return Stored::default() };
    let Ok(json) = serde_json::from_str::<Value>(&text) else {
        let _ = fs::rename(path, path.with_extension("json.abime"));
        return Stored { damaged: true, ..Stored::default() };
    };
    let items = json["rules"].as_array().cloned().unwrap_or_default();
    if json["schema_version"].as_u64().unwrap_or(1) > SCHEMA_VERSION as u64 {
        return Stored { unreadable: items, newer: true, ..Stored::default() };
    }
    let (rules, unreadable) = parse_rules(&items);
    Stored { rules, unreadable, damaged: false, newer: false }
}

/// Écrit les règles : fichier temporaire puis renommage ; les entrées illisibles sont réécrites telles quelles.
pub fn save(path: &Path, stored: &Stored) -> io::Result<()> {
    let items = stored.rules.iter().map(|r| serde_json::to_value(Rule { schema_version: SCHEMA_VERSION, ..r.clone() }).unwrap_or(Value::Null));
    let rules: Vec<Value> = items.chain(stored.unreadable.iter().cloned()).collect();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let temporary = path.with_extension("json.tmp");
    fs::write(&temporary, serde_json::to_vec_pretty(&json!({"schema_version": SCHEMA_VERSION, "rules": rules}))?)?;
    fs::rename(temporary, path)
}

/// Enregistre les règles ; une erreur d'écriture devient un refus lisible.
pub fn save_checked(path: &Path, stored: &Stored) -> Resultat<()> {
    Ok(save(path, stored)?)
}
