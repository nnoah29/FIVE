//! Interpréteur de chemins SVG pour dessiner les icônes avec cairo.

pub mod arc;
pub mod commands;
pub mod tokens;

use gtk4::cairo::Context;

use commands::Cmd;

/// Ajoute au chemin courant de cairo les commandes d'un tracé SVG.
pub fn trace(cr: &Context, cmds: &[Cmd]) {
    let mut cur = (0.0, 0.0);
    for cmd in cmds {
        match *cmd {
            Cmd::Move(x, y) => cr.move_to(x, y),
            Cmd::Line(x, y) => cr.line_to(x, y),
            Cmd::Cubic(c) => cr.curve_to(c[0], c[1], c[2], c[3], c[4], c[5]),
            Cmd::Arc { radii, rotation, large, sweep, to } => arc::arc_to(cr, cur, radii, rotation, (large, sweep), to),
            Cmd::Close => cr.close_path(),
        }
        cur = cr.current_point().unwrap_or(cur);
    }
}
