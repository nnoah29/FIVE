//! Saisies de l'éditeur de règles : tailles (« 5 Mo »), droits en octal, listes séparées par des virgules, dates locales.

use five_engine::rules::time_calc::{format_date, parse_local};

/// Lit une taille : `5 Mo`, `1,5 Go`, `300` (octets). Unités décimales (Ko = 1 000 octets) ; `None` si illisible.
pub fn parse_bytes(text: &str) -> Option<u64> {
    let clean = text.trim().replace(',', ".").replace(' ', "");
    let split = clean.find(|c: char| c.is_alphabetic()).unwrap_or(clean.len());
    let (number, unit) = clean.split_at(split);
    let factor = match unit.to_lowercase().trim_end_matches('o').trim_end_matches('b') {
        "" => 1.0,
        "k" => 1e3,
        "m" => 1e6,
        "g" => 1e9,
        "t" => 1e12,
        _ => return None,
    };
    number.parse::<f64>().ok().filter(|n| *n >= 0.0).map(|n| (n * factor).round() as u64)
}

/// Écrit une taille avec la plus grande unité entière : `5 Mo`, `1,5 Go`, `300 o`.
pub fn format_bytes(bytes: u64) -> String {
    let units = [(1_000_000_000_000.0, "To"), (1e9, "Go"), (1e6, "Mo"), (1e3, "Ko")];
    let found = units.iter().find(|(size, _)| bytes as f64 >= *size);
    found.map_or_else(
        || format!("{bytes} o"),
        |(size, name)| format!("{} {name}", format!("{:.2}", bytes as f64 / size).trim_end_matches('0').trim_end_matches('.').replace('.', ",")),
    )
}

/// Droits écrits en octal (`644`, `0755`) ; `None` au-delà de 777 (droits spéciaux refusés).
pub fn parse_octal(text: &str) -> Option<u32> {
    u32::from_str_radix(text.trim().trim_start_matches("0o"), 8).ok().filter(|mode| *mode <= 0o777)
}

/// Éléments d'une liste saisie « a, b, c » : sans espaces autour, sans éléments vides.
pub fn parse_list(text: &str) -> Vec<String> {
    text.split(',').map(str::trim).filter(|item| !item.is_empty()).map(String::from).collect()
}

/// Date locale saisie `AAAA-MM-JJ hh:mm` en secondes Unix, et l'inverse pour l'affichage.
pub fn date_text(seconds: Option<f64>) -> String {
    seconds.map_or(String::new(), |s| format_date(s, "AAAA-MM-JJ hh:mm"))
}

/// Lit une date saisie (`AAAA-MM-JJ` ou `AAAA-MM-JJ hh:mm`).
pub fn parse_date(text: &str) -> Option<f64> {
    parse_local(text)
}
