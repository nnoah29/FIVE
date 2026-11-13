//! Couleur principale : validation, contraste du texte posé dessus, pastilles proposées.

/// Pastilles proposées ; la première est la couleur d'origine.
pub const SWATCHES: [&str; 8] = ["#e8542b", "#e0434b", "#d6407a", "#8b5ce6", "#3b7bea", "#1fb5c9", "#22a667", "#e0a60e"];

/// Indique si le texte est une couleur `#rgb` ou `#rrggbb`.
pub fn is_hex(text: &str) -> bool {
    let digits = text.strip_prefix('#').unwrap_or("");
    matches!(digits.len(), 3 | 6) && digits.chars().all(|c| c.is_ascii_hexdigit())
}

/// Luminance relative (0 à 1) d'une couleur hexadécimale valide.
pub fn luminance(hex: &str) -> f64 {
    let digits: Vec<char> = hex.trim_start_matches('#').chars().collect();
    let full: Vec<char> = if digits.len() == 3 { digits.iter().flat_map(|c| [*c, *c]).collect() } else { digits };
    let channel = |at: usize| {
        let value = u8::from_str_radix(&full[at..at + 2].iter().collect::<String>(), 16).unwrap_or(0) as f64 / 255.0;
        if value <= 0.03928 { value / 12.92 } else { ((value + 0.055) / 1.055).powf(2.4) }
    };
    0.2126 * channel(0) + 0.7152 * channel(2) + 0.0722 * channel(4)
}

/// Couleur du texte posé sur la couleur principale : sombre ou blanche selon le contraste.
pub fn on_accent(hex: &str) -> &'static str {
    let l = luminance(hex);
    if (l + 0.05) / 0.061 > 1.05 / (l + 0.05) { "#1b1b1b" } else { "#ffffff" }
}
