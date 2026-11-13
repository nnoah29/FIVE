//! Lecture minimale du fichier XBEL des documents récents (`recently-used.xbel`).

/// Valeur d'un attribut XML dans une balise ouvrante, avec décodage des entités usuelles.
pub fn attribute(tag: &str, name: &str) -> Option<String> {
    let marker = format!(" {name}=\"");
    let start = tag.find(&marker)? + marker.len();
    let end = start + tag[start..].find('"')?;
    Some(decode_entities(&tag[start..end]))
}

/// Remplace les cinq entités XML prédéfinies par leur caractère.
fn decode_entities(text: &str) -> String {
    text.replace("&lt;", "<").replace("&gt;", ">").replace("&quot;", "\"").replace("&apos;", "'").replace("&amp;", "&")
}

/// Marque-pages (adresse, date de modification) du document, du plus récent au plus ancien.
pub fn bookmarks(xml: &str) -> Vec<(String, String)> {
    let mut rows: Vec<(String, String)> = xml
        .split("<bookmark ")
        .skip(1)
        .filter_map(|chunk| {
            let tag = format!(" {}", chunk.split('>').next()?);
            Some((attribute(&tag, "modified").unwrap_or_default(), attribute(&tag, "href")?))
        })
        .collect();
    rows.sort_by(|a, b| b.cmp(a));
    rows.into_iter().map(|(modified, href)| (href, modified)).collect()
}
