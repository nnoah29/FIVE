//! Surveillance du fond d'écran : la feuille de style suit les changements du fichier de couleurs.

use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

use gtk4::glib;

use super::shared::App;
use crate::ui::wallpaper::wall_stamp;

impl App {
    /// Recalcule les couleurs quand le fichier du fond d'écran change (contrôle toutes les deux secondes).
    pub fn watch_wallpaper(self: &Rc<Self>) {
        let (app, last) = (self.clone(), Cell::new(wall_stamp()));
        glib::timeout_add_local(Duration::from_secs(2), move || {
            let stamp = wall_stamp();
            if last.replace(stamp) != stamp && app.set.borrow().wallpaper {
                app.apply_theme();
            }
            glib::ControlFlow::Continue
        });
    }
}
