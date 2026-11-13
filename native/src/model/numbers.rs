//! Nombres à la française : séparateur de milliers et virgule décimale.

/// Nombre entier avec espaces fines insécables entre les milliers.
pub fn group_digits(n: u64) -> String {
    let digits = n.to_string();
    let mut out = String::new();
    for (at, ch) in digits.chars().enumerate() {
        if at > 0 && (digits.len() - at).is_multiple_of(3) {
            out.push('\u{202f}');
        }
        out.push(ch);
    }
    out
}

/// Sépare les milliers d'un nombre déjà formaté (partie entière seulement).
pub fn group_text(text: &str) -> String {
    let (whole, fraction) = text.split_once(',').map_or((text, ""), |(w, f)| (w, f));
    let grouped = group_digits(whole.parse().unwrap_or(0));
    if fraction.is_empty() { grouped } else { format!("{grouped},{fraction}") }
}
