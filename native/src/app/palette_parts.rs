//! Éléments de la palette : ligne de fichier, touches, zone des résultats.

use std::rc::Rc;

use gtk4::gdk::Key;
use gtk4::glib::Propagation;
use gtk4::prelude::*;
use gtk4::{EventControllerKey, PolicyType, ScrolledWindow};

use super::palette::{Palette, Row};
use super::shared::App;
use crate::model::kinds::{kind_icon, kind_of};
use crate::model::paths::parent_of;
use crate::ui::widgets::vbox;

/// Ligne de résultat pour un fichier ou un dossier trouvé.
pub fn file_row(item: five_engine::entry::Entry) -> Row {
    let (dir, path) = (item.is_dir, item.path.clone());
    Row {
        icon: if dir { "folder".into() } else { kind_icon(kind_of(&item)).into() },
        text: item.name.clone(),
        sub: Some(format!("/{}", parent_of(&item.path))),
        tag: if dir { "Dossier" } else { "Fichier" }.into(),
        run: Rc::new(move |a: &Rc<App>| {
            if dir {
                a.go(&path)
            } else {
                a.st.borrow_mut().pending = Some(path.clone());
                a.go(&parent_of(&path))
            }
        }),
    }
}

/// Contrôleur de touches de la palette : flèches pour choisir, Entrée pour exécuter.
pub fn key_controller(palette: &Rc<Palette>) -> EventControllerKey {
    let keys = EventControllerKey::new();
    keys.set_propagation_phase(gtk4::PropagationPhase::Capture);
    let nav = palette.clone();
    keys.connect_key_pressed(move |_, key, _, _| {
        let count = nav.rows.borrow().len();
        match key {
            Key::Down | Key::Up if count > 0 => {
                let next = (nav.index.get() as isize + if key == Key::Down { 1 } else { -1 }).clamp(0, count as isize - 1);
                nav.index.set(next as usize);
                nav.draw("");
                Propagation::Stop
            }
            Key::Return | Key::KP_Enter => {
                nav.run(nav.index.get());
                Propagation::Stop
            }
            _ => Propagation::Proceed,
        }
    });
    keys
}

/// Zone des résultats : boîte et défilement limité à la moitié de la hauteur de la fenêtre.
pub fn results_area(window_height: i32) -> (gtk4::Box, ScrolledWindow) {
    let results = vbox(0);
    results.add_css_class("results");
    let scroll = ScrolledWindow::builder()
        .hscrollbar_policy(PolicyType::Never)
        .max_content_height((window_height as f64 * 0.5) as i32)
        .propagate_natural_height(true)
        .child(&results)
        .build();
    (results, scroll)
}
