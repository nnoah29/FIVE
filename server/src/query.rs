//! Analyse de la chaîne de requête (`a=1&b=x%20y`).

use std::collections::HashMap;

use five_engine::urlcodec::decode;

/// Découpe une chaîne de requête ; `+` vaut une espace, la première valeur d'une clé l'emporte.
pub fn parse_query(raw: &str) -> HashMap<String, String> {
    let mut out = HashMap::new();
    for pair in raw.split('&').filter(|p| !p.is_empty()) {
        let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
        let clean = |text: &str| decode(&text.replace('+', " "));
        out.entry(clean(key)).or_insert_with(|| clean(value));
    }
    out
}

/// Sépare l'URL de la requête en chemin et chaîne de requête.
pub fn split_url(url: &str) -> (&str, &str) {
    url.split_once('?').unwrap_or((url, ""))
}
