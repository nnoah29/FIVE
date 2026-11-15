//! Barre latérale du sélecteur : dossier personnel, dossiers habituels, dossiers épinglés, racine.

use std::path::{Path, PathBuf};
use std::rc::Rc;

use five_engine::places::xdg_user_dirs;
use gtk4::prelude::*;

use super::window::Chooser;
use crate::model::settings::Settings;
use crate::ui::icon::icon;
use crate::ui::widgets::{hbox, label};

const USUAL: [(&str, &str, &str, &str); 6] = [
    ("desktop", "Bureau", "DESKTOP", "Desktop"),
    ("downloads", "Téléchargements", "DOWNLOAD", "Downloads"),
    ("docs", "Documents", "DOCUMENTS", "Documents"),
    ("pictures", "Images", "PICTURES", "Pictures"),
    ("music", "Musique", "MUSIC", "Music"),
    ("videos", "Vidéos", "VIDEOS", "Videos"),
];

/// Un emplacement proposé : icône, libellé, dossier.
type Place = (String, String, PathBuf);

/// Dossier personnel, dossiers habituels qui existent, puis la racine du système.
fn usual_places(home: &Path) -> Vec<Place> {
    let xdg = xdg_user_dirs(home);
    let mut out = vec![("home".to_string(), "Dossier personnel".to_string(), home.to_path_buf())];
    for (icon, name, key, default) in USUAL {
        let dir = xdg.iter().find(|(k, _)| k == key).map_or_else(|| home.join(default), |(_, v)| PathBuf::from(v));
        if dir.is_dir() && dir != home {
            out.push((icon.to_string(), name.to_string(), dir));
        }
    }
    out
}

/// Dossiers épinglés dans FIVE (barre latérale et barre du bas) qui existent encore.
fn pinned_places(home: &Path, usual: &[Place]) -> Vec<Place> {
    let settings = Settings::load(home);
    let side = settings.side_folders.unwrap_or_default().into_iter().map(|f| PathBuf::from(f.path));
    let dock = settings.dock.into_iter().map(PathBuf::from);
    let known = |p: &PathBuf| usual.iter().any(|(_, _, dir)| dir == p);
    let all: Vec<PathBuf> = side.chain(dock).filter(|p| p.is_dir() && !known(p)).collect();
    let name = |p: &Path| p.file_name().map_or_else(|| p.to_string_lossy().into_owned(), |n| n.to_string_lossy().into_owned());
    let mut out: Vec<Place> = Vec::new();
    for dir in all.into_iter().filter(|p| !out.iter().any(|(_, _, seen)| seen == p)).collect::<Vec<_>>() {
        out.push(("folder".to_string(), name(&dir), dir));
    }
    out
}

impl Chooser {
    /// Bouton d'un emplacement : icône et libellé, un clic ouvre le dossier.
    fn place_button(self: &Rc<Self>, place: &Place) -> gtk4::Button {
        let (button, row) = (gtk4::Button::new(), hbox(14));
        button.add_css_class("navitem");
        row.append(&icon(&place.0));
        let text = label(&place.1, &[]);
        text.set_ellipsize(gtk4::pango::EllipsizeMode::End);
        row.append(&text);
        button.set_child(Some(&row));
        let (weak, dir) = (Rc::downgrade(self), place.2.clone());
        button.connect_clicked(move |_| weak.upgrade().iter().for_each(|c| c.go(&dir)));
        button
    }

    /// Remplit la barre latérale : habituels, titre « Épinglés » et dossiers épinglés, racine.
    pub fn fill_sidebar(self: &Rc<Self>) {
        let home = std::env::var_os("HOME").map(PathBuf::from).unwrap_or_default();
        let usual = usual_places(&home);
        let pinned = pinned_places(&home, &usual);
        for place in usual {
            self.side.append(&self.place_button(&place));
        }
        if !pinned.is_empty() {
            self.side.append(&label("ÉPINGLÉS", &["nav-title"]));
        }
        for place in pinned {
            self.side.append(&self.place_button(&place));
        }
        let root: Place = ("drive".to_string(), "Système de fichiers".to_string(), PathBuf::from("/"));
        self.side.append(&label("ORDINATEUR", &["nav-title"]));
        self.side.append(&self.place_button(&root));
    }
}
