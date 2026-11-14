//! Date écrite dans un nom de fichier : `AAAA-MM-JJ`, `AAAAMMJJ`, `AAAA_MM_JJ`, `JJ-MM-AAAA`.

use std::sync::OnceLock;

use regex::Regex;

/// Les deux formes reconnues : année d'abord, ou jour d'abord.
fn patterns() -> &'static [Option<Regex>; 2] {
    static PATTERNS: OnceLock<[Option<Regex>; 2]> = OnceLock::new();
    PATTERNS.get_or_init(|| {
        [Regex::new(r"(?:^|\D)(\d{4})[-_.]?(\d{2})[-_.]?(\d{2})(?:\D|$)").ok(), Regex::new(r"(?:^|\D)(\d{2})[-_.](\d{2})[-_.](\d{4})(?:\D|$)").ok()]
    })
}

/// Valide une date : année 1970 à 2100, mois 1 à 12, jour 1 à 31.
fn valid(year: i32, month: u8, day: u8) -> Option<(i32, u8, u8)> {
    ((1970..=2100).contains(&year) && (1..=12).contains(&month) && (1..=31).contains(&day)).then_some((year, month, day))
}

/// Première date trouvée dans le nom (année, mois, jour).
pub fn date_in_name(name: &str) -> Option<(i32, u8, u8)> {
    let [year_first, day_first] = patterns();
    let number = |text: &str| text.parse::<i32>().unwrap_or(0);
    let first = year_first.as_ref().and_then(|re| re.captures(name)).and_then(|c| valid(number(&c[1]), number(&c[2]) as u8, number(&c[3]) as u8));
    first.or_else(|| day_first.as_ref().and_then(|re| re.captures(name)).and_then(|c| valid(number(&c[3]), number(&c[2]) as u8, number(&c[1]) as u8)))
}
