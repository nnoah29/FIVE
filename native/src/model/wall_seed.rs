//! Couleurs de départ tirées du fond d'écran : fond, texte et teintes candidates pour l'accent.

use regex::Regex;
use serde_json::Value;

use super::color::Rgb;

/// Couleurs du fond d'écran : fond, texte et teintes candidates de l'accent.
#[derive(Clone, Debug, PartialEq)]
pub struct WallSeed {
    pub bg: Rgb,
    pub fg: Rgb,
    pub candidates: Vec<Rgb>,
}

/// Teintes `--color1` à `--color14` du fichier CSS de palette.
fn css_candidates(css: &str) -> Vec<Rgb> {
    let Ok(pattern) = Regex::new(r"--color(\d+):\s*#([0-9a-fA-F]{6})") else { return Vec::new() };
    let wanted = |n: &str| n.parse::<u32>().is_ok_and(|n| (1..=14).contains(&n));
    pattern.captures_iter(css).filter(|c| wanted(&c[1])).filter_map(|c| Rgb::from_hex(&c[2])).collect()
}

/// Teintes de secours lues dans le JSON de couleurs.
fn json_candidates(json: &Value) -> Vec<Rgb> {
    let keys = ["secondary", "tertiary", "materialBlue", "materialGreen", "materialOrange", "materialRed", "materialYellow"];
    keys.iter().filter_map(|k| json[k].as_str().and_then(Rgb::from_hex)).collect()
}

/// Construit les couleurs du fond d'écran depuis le JSON `base`/`text` et le CSS de palette ; `None` si le JSON est inutilisable.
pub fn parse_seed(json: &str, css: &str) -> Option<WallSeed> {
    let json: Value = serde_json::from_str(json).ok()?;
    let bg = Rgb::from_hex(json["base"].as_str()?)?;
    let fg = Rgb::from_hex(json["text"].as_str()?)?;
    let found = css_candidates(css);
    let candidates = if found.is_empty() { json_candidates(&json) } else { found };
    Some(WallSeed { bg, fg, candidates })
}
