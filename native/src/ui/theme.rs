//! Thèmes clair et sombre : jetons de couleur, couleur principale, densité, feuille de style GTK.

use crate::model::accent::{SWATCHES, is_hex, on_accent};
use crate::model::settings::Settings;

const TEMPLATE: &str = include_str!("style.css");

/// Jetons de couleur d'un thème, au format CSS.
pub struct Palette {
    pub tokens: Vec<(&'static str, String)>,
}

/// Composantes 0–255 d'une couleur `#rgb` ou `#rrggbb`.
pub fn parse_hex(hex: &str) -> (u8, u8, u8) {
    let digits: Vec<char> = hex.trim_start_matches('#').chars().collect();
    let full: Vec<char> = if digits.len() == 3 { digits.iter().flat_map(|c| [*c, *c]).collect() } else { digits };
    let part = |at: usize| u8::from_str_radix(&full.get(at..at + 2).map_or(String::new(), |s| s.iter().collect()), 16).unwrap_or(0);
    (part(0), part(2), part(4))
}

/// Couleur `rgba(...)` d'un hexadécimal avec transparence.
pub fn rgba(hex: &str, alpha: f64) -> String {
    let (r, g, b) = parse_hex(hex);
    format!("rgba({r},{g},{b},{alpha})")
}

/// Assombrit une couleur : garde `keep` (0 à 1) de la couleur, le reste devient noir.
pub fn darken(hex: &str, keep: f64) -> String {
    let (r, g, b) = parse_hex(hex);
    let scale = |v: u8| (v as f64 * keep).round() as u8;
    format!("#{:02x}{:02x}{:02x}", scale(r), scale(g), scale(b))
}

/// Jetons du thème : `dark` choisit le sombre, `accent` est la couleur principale choisie (vide = orange d'origine).
pub fn palette(dark: bool, accent: &str) -> Palette {
    let base = if is_hex(accent) { accent.to_lowercase() } else { SWATCHES[0].to_string() };
    let custom = base != SWATCHES[0];
    let shown = if dark { base.clone() } else { darken(&base, 0.92) };
    let on = if custom {
        on_accent(&base)
    } else if dark {
        "#1b1b1b"
    } else {
        "#ffffff"
    };
    let (soft, colors) = if dark {
        (0.12, ["#1b1b1b", "#1e1e1e", "#242424", "#2a2a2a", "#333333", "#2b2b2b", "#ecebe6", "#d6d5cd", "#adaca5", "#8b8a84", "#ecebe6", "#1b1b1b"])
    } else {
        (0.10, ["#f3f1ea", "#eae7de", "#fbfaf6", "#e3e0d6", "#cdc9bd", "#dcd8cc", "#1d1c1a", "#34322e", "#4b4943", "#6b685f", "#1d1c1a", "#f3f1ea"])
    };
    let names = ["bg", "bg_side", "surface", "surface2", "line", "line_soft", "text", "body", "muted", "dim", "toast_bg", "toast_fg"];
    let mut tokens: Vec<(&'static str, String)> = names.iter().zip(colors).map(|(n, c)| (*n, c.to_string())).collect();
    tokens.push(("accent", shown.clone()));
    tokens.push(("on_accent", on.to_string()));
    tokens.push(("accent_soft", rgba(&shown, soft)));
    Palette { tokens }
}

/// Feuille de style complète : jetons remplacés, densité appliquée.
pub fn build_css(dark: bool, settings: &Settings) -> String {
    let mut css = TEMPLATE.to_string();
    let (row, kind) = if settings.compact { ("36", "5") } else { ("54", "9") };
    let mut tokens = palette(dark, &settings.accent).tokens;
    tokens.push(("row_h", row.to_string()));
    tokens.push(("kind_pad", kind.to_string()));
    tokens.push(("date_w", if settings.full_dates { "150" } else { "104" }.to_string()));
    for (name, value) in tokens {
        css = css.replace(&format!("@{name}@"), &value);
    }
    css
}
