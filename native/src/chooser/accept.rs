//! Validation du sélecteur : fichiers à ouvrir, dossiers à choisir, fichier à enregistrer.

use std::path::PathBuf;
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{AlertDialog, gio};

use super::request::Mode;
use super::window::Chooser;

impl Chooser {
    /// Valide selon le mode de la demande.
    pub fn accept(self: &Rc<Self>) {
        match self.request.mode {
            Mode::Open => self.accept_open(),
            Mode::Folder => self.accept_folder(),
            Mode::Save => self.accept_save(),
        }
    }

    /// Ouvrir : les fichiers sélectionnés ; si seuls des dossiers le sont, on entre dans le premier.
    fn accept_open(self: &Rc<Self>) {
        let selected = self.selected_rows();
        let files: Vec<PathBuf> = selected.iter().filter(|r| !r.is_dir).map(|r| r.path.clone()).collect();
        match (files.is_empty(), selected.first()) {
            (false, _) => self.finish_with(files),
            (true, Some(folder)) => self.go(&folder.path),
            (true, None) => {}
        }
    }

    /// Dossier : les dossiers sélectionnés, ou le dossier courant si rien n'est sélectionné.
    fn accept_folder(self: &Rc<Self>) {
        let mut paths: Vec<PathBuf> = self.selected_rows().iter().map(|r| r.path.clone()).collect();
        if paths.is_empty() {
            paths.push(self.dir.borrow().clone());
        }
        self.finish_with(paths);
    }

    /// Enregistrer : dossier courant plus nom saisi ; demande confirmation avant d'écraser.
    fn accept_save(self: &Rc<Self>) {
        let name = self.footer.name.text().trim().to_string();
        if name.is_empty() || name.contains('/') {
            return;
        }
        let target = self.dir.borrow().join(name);
        if target.exists() { self.confirm_overwrite(target) } else { self.finish_with(vec![target]) }
    }

    /// Demande si le fichier existant doit être remplacé.
    fn confirm_overwrite(self: &Rc<Self>, target: PathBuf) {
        let name = target.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
        let dialog = AlertDialog::builder().message(format!("Remplacer « {name} » ?")).detail("Un fichier du même nom existe déjà dans ce dossier.").build();
        dialog.set_buttons(&["Annuler", "Remplacer"]);
        dialog.set_cancel_button(0);
        dialog.set_default_button(0);
        let weak = Rc::downgrade(self);
        dialog.choose(Some(&self.window), gio::Cancellable::NONE, move |answer| {
            if let (Ok(1), Some(chooser)) = (answer, weak.upgrade()) {
                chooser.finish_with(vec![target]);
            }
        });
    }
}
