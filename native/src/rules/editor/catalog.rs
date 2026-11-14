//! Catalogue côté interface : familles, éléments d'une famille, champs, valeur par défaut d'un élément neuf.

use serde_json::{Map, Value};

use super::catalog_blank::blank_field;

/// Catalogue des conditions, actions et déclencheurs fourni par le moteur.
pub struct Catalog {
    all: Value,
}

impl Catalog {
    /// Charge le catalogue du moteur.
    pub fn load() -> Catalog {
        Catalog { all: five_engine::rules::catalog::catalog() }
    }

    /// Élément du catalogue (`conditions`, `actions` ou `triggers`) d'un type donné.
    pub fn entry(&self, list: &str, kind: &str) -> Option<&Value> {
        self.all[list].as_array()?.iter().find(|e| e["type"] == kind)
    }

    /// Familles de la liste, dans l'ordre du catalogue et sans doublon.
    pub fn families(&self, list: &str) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        for entry in self.all[list].as_array().into_iter().flatten() {
            let family = entry["family"].as_str().unwrap_or("").to_string();
            if !out.contains(&family) {
                out.push(family);
            }
        }
        out
    }

    /// Types d'une famille.
    pub fn kinds_of(&self, list: &str, family: &str) -> Vec<String> {
        self.all[list].as_array().into_iter().flatten().filter(|e| e["family"] == family).filter_map(|e| e["type"].as_str().map(String::from)).collect()
    }

    /// Famille d'un type.
    pub fn family_of(&self, list: &str, kind: &str) -> String {
        self.entry(list, kind).and_then(|e| e["family"].as_str()).unwrap_or("").to_string()
    }

    /// Élément neuf d'un type : l'exemple du catalogue, vidé de ses valeurs d'illustration.
    pub fn blank(&self, list: &str, kind: &str) -> Value {
        let Some(entry) = self.entry(list, kind) else { return Value::Null };
        let fields = entry["fields"].as_array().cloned().unwrap_or_default();
        let mut out = Map::new();
        for (key, value) in entry["example"].as_object().into_iter().flatten() {
            let field = fields.iter().find(|f| f["name"] == key.as_str()).cloned().unwrap_or(Value::Null);
            out.insert(key.clone(), if key == "type" { value.clone() } else { blank_field(key, &field, value) });
        }
        Value::Object(out)
    }
}
