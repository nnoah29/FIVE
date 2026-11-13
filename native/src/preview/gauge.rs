//! Jauge en demi-cercle du panneau Espace.

use std::f64::consts::PI;

use gtk4::cairo::LineCap;
use gtk4::prelude::*;
use gtk4::{Align, DrawingArea, Overlay};

use crate::ui::widgets::{label, vbox};

/// Demi-cercle de la grille 240 × 124, du début jusqu'à `ratio` ; la couleur vient du CSS du widget.
fn arc_layer(class: &str, ratio: f64) -> DrawingArea {
    let area = DrawingArea::new();
    area.set_content_width(312);
    area.set_content_height(161);
    area.add_css_class(class);
    area.set_draw_func(move |widget, cr, width, _| {
        let color = widget.color();
        cr.set_source_rgba(color.red() as f64, color.green() as f64, color.blue() as f64, color.alpha() as f64);
        let scale = width as f64 / 240.0;
        cr.scale(scale, scale);
        cr.set_line_width(26.0);
        cr.set_line_cap(LineCap::Butt);
        cr.arc(120.0, 118.0, 100.0, PI, PI + ratio.clamp(0.0, 1.0) * PI);
        let _ = cr.stroke();
    });
    area
}

/// Jauge complète : piste, partie utilisée, grand chiffre et légende.
pub fn gauge(ratio: f64, big: &str, caption: &str) -> Overlay {
    let overlay = Overlay::new();
    overlay.set_child(Some(&arc_layer("gauge-track", 1.0)));
    overlay.add_overlay(&arc_layer("gauge-used", ratio));
    let read = vbox(6);
    read.set_halign(Align::Center);
    read.set_valign(Align::End);
    read.set_margin_bottom(6);
    let number = label(big, &["gauge-read-b"]);
    number.set_halign(Align::Center);
    let note = label(caption, &["gauge-read-s"]);
    note.set_halign(Align::Center);
    read.append(&number);
    read.append(&note);
    overlay.add_overlay(&read);
    overlay
}
