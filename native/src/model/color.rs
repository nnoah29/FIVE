//! Couleur RVB en flottants : lecture hexadécimale, mélange, luminance et contraste.

/// Couleur dont les composantes vont de 0 à 1.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rgb {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}

impl Rgb {
    /// Lit `#rrggbb` (le dièse est facultatif) ; `None` si le texte n'est pas une couleur.
    pub fn from_hex(text: &str) -> Option<Rgb> {
        let digits = text.trim().trim_start_matches('#');
        let digits = digits.get(..6)?;
        let part = |at: usize| u8::from_str_radix(&digits[at..at + 2], 16).ok().map(|v| v as f64 / 255.0);
        Some(Rgb { r: part(0)?, g: part(2)?, b: part(4)? })
    }

    /// Écrit la couleur en `#rrggbb`.
    pub fn to_hex(self) -> String {
        let byte = |v: f64| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
        format!("#{:02x}{:02x}{:02x}", byte(self.r), byte(self.g), byte(self.b))
    }

    /// Écrit la couleur en `rgba(...)` avec la transparence donnée.
    pub fn to_rgba(self, alpha: f64) -> String {
        let byte = |v: f64| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
        format!("rgba({},{},{},{alpha})", byte(self.r), byte(self.g), byte(self.b))
    }

    /// Mélange : `t` = 0 redonne la couleur, `t` = 1 donne `other`.
    pub fn mix(self, other: Rgb, t: f64) -> Rgb {
        Rgb { r: self.r + (other.r - self.r) * t, g: self.g + (other.g - self.g) * t, b: self.b + (other.b - self.b) * t }
    }

    /// Luminance relative (0 à 1) selon la norme WCAG.
    pub fn luminance(self) -> f64 {
        let linear = |v: f64| if v <= 0.03928 { v / 12.92 } else { ((v + 0.055) / 1.055).powf(2.4) };
        0.2126 * linear(self.r) + 0.7152 * linear(self.g) + 0.0722 * linear(self.b)
    }

    /// Rapport de contraste WCAG entre deux couleurs (1 à 21).
    pub fn contrast(self, other: Rgb) -> f64 {
        let (a, b) = (self.luminance(), other.luminance());
        (a.max(b) + 0.05) / (a.min(b) + 0.05)
    }
}
