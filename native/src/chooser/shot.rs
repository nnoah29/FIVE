//! Capture d'écran du sélecteur après un court délai, pour vérifier son aspect sans outil externe.

use std::rc::Rc;
use std::time::Duration;

use gtk4::glib;

use super::window::Chooser;
use crate::ui::capture::save_png;

/// Programme la capture dans `file` (puis la fermeture) si un fichier est demandé.
pub fn schedule(chooser: &Rc<Chooser>, file: Option<String>) {
    let Some(file) = file else { return };
    let chooser = chooser.clone();
    glib::timeout_add_local_once(Duration::from_millis(1200), move || {
        save_png(&chooser.window, &file);
        chooser.cancel();
    });
}
