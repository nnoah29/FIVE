//! Capture de la fenêtre en PNG, sans dépendre d'un outil externe (vérification visuelle).

use gtk4::prelude::*;
use gtk4::{ApplicationWindow, Snapshot, WidgetPaintable};

/// Enregistre l'aspect actuel de la fenêtre dans un fichier PNG ; `false` si la capture échoue.
pub fn save_png(window: &ApplicationWindow, path: &str) -> bool {
    let paintable = WidgetPaintable::new(Some(window));
    let snapshot = Snapshot::new();
    paintable.snapshot(&snapshot, window.width() as f64, window.height() as f64);
    let renderer = window.native().and_then(|native| native.renderer());
    match (snapshot.to_node(), renderer) {
        (Some(node), Some(renderer)) => renderer.render_texture(&node, None).save_to_png(path).is_ok(),
        _ => false,
    }
}
