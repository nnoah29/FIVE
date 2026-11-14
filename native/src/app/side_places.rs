//! Dossiers de la section « Dossiers » de la barre latérale : automatiques (XDG) ou choisis par l'utilisateur.

use std::path::Path;
use std::rc::Rc;

use five_engine::entry::Entry;
use five_engine::realpath::real_path;

use super::shared::App;
use crate::model::side_folders::{SideFolder, pin_folder, shown_label};
use crate::state::Place;

impl App {
    /// Dossiers détectés sous la racine (Téléchargements, Images…), avec leur icône, sans la corbeille.
    pub fn default_side_folders(&self) -> Vec<SideFolder> {
        let places = self.st.borrow().places.clone();
        let absolute = |rel: &str| self.root.path().join(rel).to_string_lossy().into_owned();
        places.iter().filter(|p| p.id != "trash").map(|p| SideFolder { label: p.label.clone(), path: absolute(&p.path), icon: p.id.clone() }).collect()
    }

    /// Dossiers affichés : la liste choisie (ceux qui n'existent plus ou sont hors racine sont masqués), sinon les dossiers XDG.
    pub fn side_places(&self) -> Vec<Place> {
        let Some(list) = self.set.borrow().side_folders.clone() else {
            return self.st.borrow().places.iter().filter(|p| p.id != "trash").cloned().collect();
        };
        let usable = |full: &Path| self.root.inside(full) && full.is_dir() && full != self.root.path();
        let place = |f: &SideFolder| Place { id: f.icon.clone(), label: shown_label(f), path: self.root.relative(&real_path(Path::new(&f.path))) };
        list.iter().filter(|f| usable(&real_path(Path::new(&f.path)))).map(place).collect()
    }

    /// Modifie la liste choisie ; le mode automatique devient personnalisé en partant des dossiers par défaut.
    pub fn edit_side_folders(self: &Rc<Self>, edit: impl FnOnce(&mut Vec<SideFolder>)) {
        let defaults = self.default_side_folders();
        self.change_setting(|s| edit(s.side_folders.get_or_insert(defaults)));
    }

    /// Épingle le dossier de `path` (relatif à la racine) dans la barre latérale.
    pub fn pin_side_folder(self: &Rc<Self>, path: &str) {
        let Ok(full) = self.root.resolve(path) else { return };
        let (text, mut added) = (full.to_string_lossy().into_owned(), false);
        self.edit_side_folders(|list| added = pin_folder(list, &text));
        self.toast(if added { "Dossier épinglé dans la barre latérale" } else { "Ce dossier est déjà dans la barre latérale" });
    }

    /// Épingle un dossier de la liste ; refuse un fichier.
    pub fn pin_entry(self: &Rc<Self>, item: &Entry) {
        if item.is_dir { self.pin_side_folder(&item.path) } else { self.toast("Seuls les dossiers peuvent être épinglés") }
    }
}
