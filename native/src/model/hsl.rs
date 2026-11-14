//! Conversion entre RVB et teinte, saturation, luminosité.

use super::color::Rgb;

/// Couleur en teinte (0 à 360), saturation et luminosité (0 à 1).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hsl {
    pub h: f64,
    pub s: f64,
    pub l: f64,
}

impl Rgb {
    /// Convertit en teinte, saturation, luminosité.
    pub fn to_hsl(self) -> Hsl {
        let (max, min) = (self.r.max(self.g).max(self.b), self.r.min(self.g).min(self.b));
        let (l, d) = ((max + min) / 2.0, max - min);
        if d == 0.0 {
            return Hsl { h: 0.0, s: 0.0, l };
        }
        let s = d / (1.0 - (2.0 * l - 1.0).abs());
        let h = if max == self.r {
            ((self.g - self.b) / d).rem_euclid(6.0)
        } else if max == self.g {
            (self.b - self.r) / d + 2.0
        } else {
            (self.r - self.g) / d + 4.0
        };
        Hsl { h: h * 60.0, s, l }
    }
}

impl Hsl {
    /// Convertit en RVB.
    pub fn to_rgb(self) -> Rgb {
        let c = (1.0 - (2.0 * self.l - 1.0).abs()) * self.s;
        let x = c * (1.0 - ((self.h / 60.0).rem_euclid(2.0) - 1.0).abs());
        let m = self.l - c / 2.0;
        let (r, g, b) = match (self.h.rem_euclid(360.0) / 60.0) as u32 {
            0 => (c, x, 0.0),
            1 => (x, c, 0.0),
            2 => (0.0, c, x),
            3 => (0.0, x, c),
            4 => (x, 0.0, c),
            _ => (c, 0.0, x),
        };
        Rgb { r: r + m, g: g + m, b: b + m }
    }
}
