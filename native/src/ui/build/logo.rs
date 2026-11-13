//! Logo de FIVE : cadre ouvert en dégradé et lettre F.

use gtk4::cairo::{Context, LinearGradient};
use gtk4::prelude::*;
use gtk4::{Align, DrawingArea, Overlay};

use crate::ui::widgets::label;

/// Dessine le cadre ouvert du logo, en dégradé autour de la couleur CSS du widget.
fn draw_frame(widget: &DrawingArea, cr: &Context, width: i32, height: i32) {
    let color = widget.color();
    let (r, g, b) = (color.red() as f64, color.green() as f64, color.blue() as f64);
    cr.scale(width as f64 / 200.0, height as f64 / 200.0);
    let gradient = LinearGradient::new(26.0, 30.0, 174.0, 170.0);
    gradient.add_color_stop_rgb(0.0, r * 0.7 + 0.3, g * 0.7 + 0.3, b * 0.7 + 0.3);
    gradient.add_color_stop_rgb(0.55, r, g, b);
    gradient.add_color_stop_rgb(1.0, r * 0.8, g * 0.8, b * 0.8);
    let _ = cr.set_source(&gradient);
    cr.set_line_width(9.0);
    cr.set_miter_limit(10.0);
    cr.move_to(30.0, 118.0);
    cr.line_to(30.0, 30.0);
    cr.line_to(170.0, 30.0);
    cr.line_to(170.0, 170.0);
    cr.line_to(82.0, 170.0);
    let _ = cr.stroke();
}

/// Logo de 48 pixels : cadre dessiné sous un F en Anton, tous deux à la couleur principale.
pub fn logo() -> Overlay {
    let frame = DrawingArea::new();
    frame.set_content_width(48);
    frame.set_content_height(48);
    frame.add_css_class("logo");
    frame.set_draw_func(draw_frame);
    let letter = label("F", &["logo-f"]);
    letter.set_halign(Align::Center);
    letter.set_valign(Align::Center);
    letter.set_margin_top(6);
    let overlay = Overlay::new();
    overlay.set_child(Some(&frame));
    overlay.add_overlay(&letter);
    overlay
}
