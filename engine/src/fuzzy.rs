//! Recherche floue : les lettres de la requête doivent apparaître dans l'ordre.

/// Score (plus petit = meilleur) si la requête est une sous-suite du nom, sinon `None`.
pub fn fuzzy_score(query: &str, name: &str) -> Option<i64> {
    let query: Vec<char> = query.to_lowercase().chars().collect();
    let name: Vec<char> = name.to_lowercase().chars().collect();
    if let Some(start) = name.windows(query.len().max(1)).position(|w| w == query.as_slice()) {
        return Some(start as i64);
    }
    let (mut at, mut gaps, mut first): (Option<usize>, i64, Option<usize>) = (None, 0, None);
    for letter in &query {
        let from = at.map_or(0, |p| p + 1);
        let found = from + name.get(from..)?.iter().position(|c| c == letter)?;
        first.get_or_insert(found);
        gaps += at.map_or(0, |p| (found - p - 1) as i64);
        at = Some(found);
    }
    Some(1000 + gaps * 4 + first.unwrap_or(0) as i64)
}
