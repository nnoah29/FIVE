//! Opérations pures sur un nom de fichier : casse, accents, espaces, remplacement, extension.

use regex::Regex;

/// Découpe un nom en radical et extension (avec le point) ; un nom caché n'a pas d'extension.
pub fn split_name(name: &str) -> (String, String) {
    match name.rfind('.') {
        Some(at) if at > 0 => (name[..at].to_string(), name[at..].to_string()),
        _ => (name.to_string(), String::new()),
    }
}

/// Retire les accents des lettres françaises et latines courantes.
pub fn strip_accents(text: &str) -> String {
    const GROUPS: [(&str, &str); 20] = [
        ("àâäáãå", "a"),
        ("ç", "c"),
        ("éèêë", "e"),
        ("íìîï", "i"),
        ("ñ", "n"),
        ("óòôöõ", "o"),
        ("úùûü", "u"),
        ("ýÿ", "y"),
        ("œ", "oe"),
        ("æ", "ae"),
        ("ÀÂÄÁÃÅ", "A"),
        ("Ç", "C"),
        ("ÉÈÊË", "E"),
        ("ÍÌÎÏ", "I"),
        ("Ñ", "N"),
        ("ÓÒÔÖÕ", "O"),
        ("ÚÙÛÜ", "U"),
        ("Ý", "Y"),
        ("Œ", "OE"),
        ("Æ", "AE"),
    ];
    text.chars().map(|c| GROUPS.iter().find(|(from, _)| from.contains(c)).map_or_else(|| c.to_string(), |(_, to)| to.to_string())).collect()
}

/// Radical sans accents, en minuscules, sans espaces ni signes (tirets), extension en minuscules.
pub fn slugify(name: &str) -> String {
    let (stem, ext) = split_name(name);
    let mut slug = String::new();
    for c in strip_accents(&stem).to_lowercase().chars() {
        if c.is_ascii_alphanumeric() {
            slug.push(c);
        } else if !slug.ends_with('-') && !slug.is_empty() {
            slug.push('-');
        }
    }
    format!("{}{}", slug.trim_end_matches('-'), ext.to_lowercase())
}

/// Change la casse du radical : minuscules, majuscules, chaque mot, ou première lettre seulement.
pub fn change_case(name: &str, mode: super::types_action::CaseMode) -> String {
    use super::types_action::CaseMode as M;
    let (stem, ext) = split_name(name);
    let word = |w: &str| w.chars().next().map_or(String::new(), |f| f.to_uppercase().collect::<String>() + &w[f.len_utf8()..].to_lowercase());
    let cased = match mode {
        M::Lower => stem.to_lowercase(),
        M::Upper => stem.to_uppercase(),
        M::Title => stem.split(' ').map(word).collect::<Vec<_>>().join(" "),
        M::Sentence => word(&stem),
    };
    cased + &ext
}

/// Remplace un texte (ou une expression régulière) dans le radical ; un motif invalide laisse le nom intact.
pub fn replace_in_name(name: &str, find: &str, replace: &str, regex: bool) -> String {
    let (stem, ext) = split_name(name);
    if find.is_empty() {
        return name.to_string();
    }
    let replaced = if regex { Regex::new(find).map_or(stem.clone(), |re| re.replace_all(&stem, replace).into_owned()) } else { stem.replace(find, replace) };
    replaced + &ext
}

/// Extension normalisée : minuscules, `jpeg` devient `jpg`, `htm` devient `html`, `tiff` devient `tif`.
pub fn normalize_extension(name: &str) -> String {
    let (stem, ext) = split_name(name);
    let lower = ext.to_lowercase();
    let fixed = match lower.as_str() {
        ".jpeg" => ".jpg",
        ".htm" => ".html",
        ".tiff" => ".tif",
        other => other,
    };
    stem + fixed
}
