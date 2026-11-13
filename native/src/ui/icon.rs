//! Icône dessinée avec cairo : prend la couleur CSS (`color`) du widget qui la porte.

use gtk4::cairo::{Context, LineCap, LineJoin};
use gtk4::prelude::*;
use gtk4::DrawingArea;

use super::icons_data::ICONS;
use super::svgpath::{commands::parse, trace};

/// Dessine les formes d'une icône sur une grille 24 × 24, dans le contexte cairo actuel.
fn draw_shapes(cr: &Context, shapes: &[&str]) {
    for shape in shapes {
        if let Some(circle) = shape.strip_prefix("c:") {
            let v: Vec<f64> = circle.split(',').filter_map(|n| n.parse().ok()).collect();
            if let [cx, cy, r] = v[..] {
                cr.new_sub_path();
                cr.arc(cx, cy, r, 0.0, std::f64::consts::TAU);
            }
        } else {
            trace(cr, &parse(shape));
        }
    }
}

/// Icône carrée de `size` pixels, au trait `stroke` (1,6 pour l'interface courante).
pub fn icon_with_stroke(name: &str, size: i32, stroke: f64) -> DrawingArea {
    let area = DrawingArea::new();
    area.set_content_width(size);
    area.set_content_height(size);
    area.set_halign(gtk4::Align::Center);
    area.set_valign(gtk4::Align::Center);
    area.add_css_class("ic");
    let shapes: Vec<&'static str> = ICONS.iter().find(|(n, _)| *n == name).map(|(_, s)| s.to_vec()).unwrap_or_default();
    area.set_draw_func(move |widget, cr, width, height| {
        let color = widget.color();
        cr.set_source_rgba(color.red() as f64, color.green() as f64, color.blue() as f64, color.alpha() as f64);
        cr.scale(width as f64 / 24.0, height as f64 / 24.0);
        cr.set_line_width(stroke);
        cr.set_line_cap(LineCap::Round);
        cr.set_line_join(LineJoin::Round);
        draw_shapes(cr, &shapes);
        let _ = cr.stroke();
    });
    area
}

/// Icône de 16 pixels au trait courant.
pub fn icon(name: &str) -> DrawingArea {
    icon_with_stroke(name, 16, 1.6)
}
