//! Filtres de noms : motifs `*.png` (avec `?` et `[abc]`) et types MIME (`image/*`).

use super::request::{Filter, FilterRule};

/// Compare un nom à un motif (`*`, `?`, ensembles `[abc]`), sans tenir compte de la casse.
pub fn glob_match(pattern: &[char], name: &[char]) -> bool {
    match (pattern.first(), name.first()) {
        (None, None) => true,
        (Some('*'), _) => glob_match(&pattern[1..], name) || (!name.is_empty() && glob_match(pattern, &name[1..])),
        (Some('?'), Some(_)) => glob_match(&pattern[1..], &name[1..]),
        (Some('['), Some(c)) => class_match(pattern, *c).is_some_and(|rest| glob_match(rest, &name[1..])),
        (Some(p), Some(c)) => p.eq_ignore_ascii_case(c) && glob_match(&pattern[1..], &name[1..]),
        _ => false,
    }
}

/// Vérifie `c` contre l'ensemble `[...]` en tête de `pattern` ; retourne la suite du motif si `c` en fait partie.
fn class_match(pattern: &[char], c: char) -> Option<&[char]> {
    let end = pattern.iter().position(|p| *p == ']')?;
    let found = pattern[1..end].iter().any(|p| p.eq_ignore_ascii_case(&c));
    found.then(|| &pattern[end + 1..])
}

/// Vérifie un type MIME, exact ou par famille (`image/*`).
fn mime_match(pattern: &str, mime: &str) -> bool {
    match pattern.strip_suffix("/*") {
        Some(family) => mime.split('/').next() == Some(family),
        None => pattern == mime,
    }
}

/// Indique si un nom de fichier passe le filtre ; un filtre sans règle accepte tout.
pub fn filter_accepts(filter: &Filter, name: &str) -> bool {
    let chars: Vec<char> = name.chars().collect();
    let mime = five_engine::mime::guess(name);
    let one = |rule: &FilterRule| match rule.kind {
        0 => glob_match(&rule.pattern.chars().collect::<Vec<_>>(), &chars),
        _ => mime_match(&rule.pattern, mime),
    };
    filter.rules.is_empty() || filter.rules.iter().any(one)
}
