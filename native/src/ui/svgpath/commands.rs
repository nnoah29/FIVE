//! Chemin SVG converti en commandes absolues.

use super::tokens::Scanner;

/// Commande de tracé en coordonnées absolues.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Cmd {
    Move(f64, f64),
    Line(f64, f64),
    Cubic([f64; 6]),
    Arc { radii: (f64, f64), rotation: f64, large: bool, sweep: bool, to: (f64, f64) },
    Close,
}

/// Lit `count` nombres à la suite.
fn numbers(scan: &mut Scanner, count: usize) -> Option<Vec<f64>> {
    (0..count).map(|_| scan.number()).collect()
}

/// Lit les arguments d'une commande et retourne la commande absolue ; `cur` est le point courant, `reflect` le point de contrôle réfléchi.
fn read_args(letter: char, scan: &mut Scanner, cur: (f64, f64), reflect: (f64, f64)) -> Option<Cmd> {
    let rel = letter.is_ascii_lowercase();
    let (ox, oy) = if rel { cur } else { (0.0, 0.0) };
    Some(match letter.to_ascii_uppercase() {
        'M' => numbers(scan, 2).map(|n| Cmd::Move(ox + n[0], oy + n[1]))?,
        'L' => numbers(scan, 2).map(|n| Cmd::Line(ox + n[0], oy + n[1]))?,
        'H' => Cmd::Line(ox + scan.number()?, cur.1),
        'V' => Cmd::Line(cur.0, oy + scan.number()?),
        'C' => numbers(scan, 6).map(|n| Cmd::Cubic([ox + n[0], oy + n[1], ox + n[2], oy + n[3], ox + n[4], oy + n[5]]))?,
        'S' => numbers(scan, 4).map(|n| Cmd::Cubic([reflect.0, reflect.1, ox + n[0], oy + n[1], ox + n[2], oy + n[3]]))?,
        'A' => {
            numbers(scan, 7).map(|n| Cmd::Arc { radii: (n[0], n[1]), rotation: n[2], large: n[3] != 0.0, sweep: n[4] != 0.0, to: (ox + n[5], oy + n[6]) })?
        }
        _ => return None,
    })
}

/// Point d'arrivée d'une commande de tracé (la fermeture revient au début du sous-chemin).
fn end_of(cmd: &Cmd, start: (f64, f64)) -> (f64, f64) {
    match cmd {
        Cmd::Move(x, y) | Cmd::Line(x, y) => (*x, *y),
        Cmd::Cubic(c) => (c[4], c[5]),
        Cmd::Arc { to, .. } => *to,
        Cmd::Close => start,
    }
}

/// Lettre de commande des répétitions : après un déplacement, les paires suivantes sont des lignes.
fn implicit_next(letter: char) -> char {
    match letter {
        'M' => 'L',
        'm' => 'l',
        other => other,
    }
}

/// Convertit le texte d'un chemin en commandes absolues (M, L, H, V, C, A, Z et leurs formes relatives).
pub fn parse(d: &str) -> Vec<Cmd> {
    let (mut out, mut scan) = (Vec::new(), Scanner::new(d));
    let (mut cur, mut start, mut ctrl) = ((0.0, 0.0), (0.0, 0.0), (0.0, 0.0));
    while let Some(letter) = scan.command() {
        if letter.eq_ignore_ascii_case(&'z') {
            out.push(Cmd::Close);
            cur = start;
            continue;
        }
        let mut letter = letter;
        while let Some(cmd) = read_args(letter, &mut scan, cur, (2.0 * cur.0 - ctrl.0, 2.0 * cur.1 - ctrl.1)) {
            ctrl = if let Cmd::Cubic(c) = cmd { (c[2], c[3]) } else { end_of(&cmd, start) };
            cur = end_of(&cmd, start);
            if matches!(cmd, Cmd::Move(..)) {
                start = cur;
            }
            out.push(cmd);
            letter = implicit_next(letter);
            if !scan.has_number() {
                break;
            }
        }
    }
    out
}
