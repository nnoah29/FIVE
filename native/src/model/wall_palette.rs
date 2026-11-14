//! Palette de l'interface calculée depuis le fond d'écran, avec les mêmes ratios que MIRA.

use super::color::Rgb;
use super::hsl::Hsl;
use super::wall_seed::WallSeed;

const WHITE: Rgb = Rgb { r: 1.0, g: 1.0, b: 1.0 };
const BLACK: Rgb = Rgb { r: 0.0, g: 0.0, b: 0.0 };
const INK: Rgb = Rgb { r: 17.0 / 255.0, g: 17.0 / 255.0, b: 17.0 / 255.0 };

/// Éclaircit ou assombrit `color` (vers blanc ou noir selon le fond) jusqu'au contraste `min`.
pub fn ensure_contrast(color: Rgb, bg: Rgb, min: f64) -> Rgb {
    if color.contrast(bg) >= min {
        return color;
    }
    let target = if bg.luminance() < 0.4 { WHITE } else { BLACK };
    (1..=20).map(|i| color.mix(target, i as f64 / 20.0)).find(|out| out.contrast(bg) >= min).unwrap_or(target)
}

/// Choisit la teinte la plus vivante du fond d'écran, rendue lisible ; le texte si le fond est presque gris.
fn pick_accent(candidates: &[Rgb], bg: Rgb, fg: Rgb, dark: bool) -> Rgb {
    let score = |h: &Hsl| h.s * (1.0 - (h.l - 0.5).abs() * 1.2).clamp(0.1, 1.0);
    let best = candidates.iter().map(|c| c.to_hsl()).filter(|h| score(h) > 0.0).max_by(|a, b| score(a).total_cmp(&score(b)));
    let Some(best) = best.filter(|h| h.s >= 0.06) else { return fg };
    let l = if dark { best.l.clamp(0.56, 0.68) } else { best.l.clamp(0.36, 0.46) };
    ensure_contrast(Hsl { h: best.h, s: best.s.clamp(0.52, 0.86), l }.to_rgb(), bg, 3.2)
}

/// Jetons de couleur de la feuille de style pour ce fond d'écran, et indication de thème sombre.
pub fn wall_tokens(seed: &WallSeed) -> (Vec<(&'static str, String)>, bool) {
    let (bg, fg) = (seed.bg, seed.fg);
    let dark = bg.luminance() < 0.4;
    let text = ensure_contrast(fg, bg, 10.0);
    let muted = ensure_contrast(text.mix(bg, 0.25), bg, 6.5);
    let dim = ensure_contrast(text.mix(bg, 0.45), bg, 4.5);
    let accent = pick_accent(&seed.candidates, bg, text, dark);
    let on = if accent.contrast(INK) >= accent.contrast(WHITE) { INK } else { WHITE };
    let hex = |c: Rgb| c.to_hex();
    let tokens = vec![
        ("bg", hex(bg)),
        ("bg_side", hex(bg.mix(fg, 0.03))),
        ("surface", hex(bg.mix(fg, 0.06))),
        ("surface2", hex(bg.mix(fg, 0.09))),
        ("line", hex(bg.mix(fg, 0.16))),
        ("line_soft", hex(bg.mix(fg, 0.10))),
        ("text", hex(text)),
        ("body", hex(text.mix(bg, 0.1))),
        ("muted", hex(muted)),
        ("dim", hex(dim)),
        ("toast_bg", hex(text)),
        ("toast_fg", hex(bg)),
        ("accent", hex(accent)),
        ("on_accent", hex(on)),
        ("accent_soft", accent.to_rgba(if dark { 0.12 } else { 0.10 })),
    ];
    (tokens, dark)
}
