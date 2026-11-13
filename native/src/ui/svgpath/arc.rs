//! Arc elliptique SVG tracé avec cairo (conversion extrémités vers centre).

use gtk4::cairo::Context;

/// Trace l'arc SVG de `from` à `to` ; rayons nuls ou points confondus donnent une ligne droite.
pub fn arc_to(cr: &Context, from: (f64, f64), radii: (f64, f64), rotation_deg: f64, flags: (bool, bool), to: (f64, f64)) {
    let (mut rx, mut ry) = (radii.0.abs(), radii.1.abs());
    if rx == 0.0 || ry == 0.0 || from == to {
        cr.line_to(to.0, to.1);
        return;
    }
    let phi = rotation_deg.to_radians();
    let (cos, sin) = (phi.cos(), phi.sin());
    let (dx, dy) = ((from.0 - to.0) / 2.0, (from.1 - to.1) / 2.0);
    let (x1, y1) = (cos * dx + sin * dy, -sin * dx + cos * dy);
    let scale = (x1 * x1) / (rx * rx) + (y1 * y1) / (ry * ry);
    if scale > 1.0 {
        rx *= scale.sqrt();
        ry *= scale.sqrt();
    }
    let numerator = rx * rx * ry * ry - rx * rx * y1 * y1 - ry * ry * x1 * x1;
    let denominator = rx * rx * y1 * y1 + ry * ry * x1 * x1;
    let sign = if flags.0 == flags.1 { -1.0 } else { 1.0 };
    let factor = sign * (numerator / denominator).max(0.0).sqrt();
    let (cx1, cy1) = (factor * rx * y1 / ry, -factor * ry * x1 / rx);
    let (cx, cy) = (cos * cx1 - sin * cy1 + (from.0 + to.0) / 2.0, sin * cx1 + cos * cy1 + (from.1 + to.1) / 2.0);
    let start = ((y1 - cy1) / ry).atan2((x1 - cx1) / rx);
    let end = ((-y1 - cy1) / ry).atan2((-x1 - cx1) / rx);
    draw_ellipse(cr, (cx, cy), (rx, ry, phi), (start, end), flags.1);
}

/// Dessine la portion d'ellipse entre deux angles, dans le sens demandé.
fn draw_ellipse(cr: &Context, center: (f64, f64), shape: (f64, f64, f64), angles: (f64, f64), sweep: bool) {
    let (rx, ry, phi) = shape;
    let (start, mut end) = angles;
    if sweep && end < start {
        end += std::f64::consts::TAU;
    }
    if !sweep && end > start {
        end -= std::f64::consts::TAU;
    }
    let saved = cr.matrix();
    cr.translate(center.0, center.1);
    cr.rotate(phi);
    cr.scale(rx, ry);
    if sweep {
        cr.arc(0.0, 0.0, 1.0, start, end)
    } else {
        cr.arc_negative(0.0, 0.0, 1.0, start, end)
    }
    cr.set_matrix(saved);
}
