//! Actions sur les fichiers : créer un dossier, mettre à la corbeille, renommer, copier le chemin.

use std::rc::Rc;

use five_engine::delete::delete_entry;
use five_engine::entry::Entry;
use five_engine::folders::make_folder;
use five_engine::rename::rename_many;
use five_engine::root::Root;

use super::dialog::{DialogSpec, Submit};
use super::shared::App;
use super::work::spawn_work;
use crate::model::paths::{in_trash, parent_of};

/// Retourne le premier message d'erreur d'une suite de résultats.
fn first_error(results: Vec<Result<(), String>>) -> Result<(), String> {
    results.into_iter().collect::<Result<Vec<_>, _>>().map(|_| ())
}

/// Met chaque chemin à la corbeille (ou l'efface s'il y est déjà), en s'arrêtant à la première erreur.
fn delete_all(root: &Root, paths: &[String]) -> Result<(), String> {
    first_error(paths.iter().map(|p| delete_entry(root, p).map(|_| ()).map_err(|r| r.message)).collect())
}

impl App {
    /// Ouvre le dialogue de création d'un dossier dans le dossier courant.
    pub fn create_folder(self: &Rc<Self>) {
        if !self.st.borrow().can_create() {
            return self.toast("Ouvrez un dossier ordinaire pour y créer un dossier");
        }
        let path = self.st.borrow().path.clone();
        let submit: Submit = Rc::new(|app, name, report| {
            let (root, parent, app) = (app.root.clone(), app.st.borrow().path.clone(), app.clone());
            spawn_work(move || make_folder(&root, &parent, &name.unwrap_or_default()).map(|_| ()).map_err(|r| r.message), move |result| {
                if result.is_ok() {
                    app.toast("Dossier créé");
                    app.reload();
                }
                report(result);
            });
        });
        self.dialog(DialogSpec { title: "Nouveau dossier".into(), text: Some(format!("Il sera créé dans /{path}")), names: None, input: Some(String::new()), confirm: "Créer".into(), submit });
    }

    /// Supprime des éléments après confirmation (la corbeille sans confirmation si le réglage le permet).
    pub fn delete_items(self: &Rc<Self>, items: Vec<Entry>) {
        if items.is_empty() {
            return;
        }
        let paths: Vec<String> = items.iter().map(|i| i.path.clone()).collect();
        let definitive = paths.iter().all(|p| in_trash(p));
        let submit: Submit = Rc::new(move |app, _, report| {
            let (root, list, app) = (app.root.clone(), paths.clone(), app.clone());
            spawn_work(move || delete_all(&root, &list), move |result| {
                if result.is_ok() {
                    app.toast(if definitive { "Supprimé définitivement" } else { "Mis à la corbeille" });
                    app.st.borrow_mut().sel = -1;
                    app.reload();
                }
                report(result);
            });
        });
        if !definitive && !self.set.borrow().confirm_trash {
            let (app, reporter) = (self.clone(), self.clone());
            return submit(&app, None, Box::new(move |r| if let Err(m) = r { reporter.toast(&m) }));
        }
        let (title, text, confirm) = if definitive { ("Supprimer définitivement ?", "Cette action est irréversible.", "Supprimer définitivement") } else { ("Mettre à la corbeille ?", "Vous pourrez le récupérer depuis la corbeille de votre système.", "Mettre à la corbeille") };
        let names = items.iter().map(|i| format!("/{}", i.path)).collect();
        self.dialog(DialogSpec { title: title.into(), text: Some(text.into()), names: Some(names), input: None, confirm: confirm.into(), submit });
    }

    /// Renomme un élément (dialogue) ou plusieurs (renommage par lot).
    pub fn rename_items(self: &Rc<Self>, items: Vec<Entry>) {
        if items.is_empty() {
            return self.toast("Sélectionnez au moins un élément");
        }
        if items.iter().any(|i| in_trash(&i.path)) {
            return self.toast("Impossible de renommer dans la corbeille");
        }
        if items.len() > 1 {
            return self.batch_rename(items);
        }
        let item = items[0].clone();
        let path = item.path.clone();
        let submit: Submit = Rc::new(move |app, name, report| {
            let (root, pair, app) = (app.root.clone(), (path.clone(), name.unwrap_or_default()), app.clone());
            spawn_work(move || rename_many(&root, &[pair]).map(|_| ()).map_err(|r| r.message), move |result| {
                if result.is_ok() {
                    app.toast("Renommé");
                    app.reload();
                }
                report(result);
            });
        });
        self.dialog(DialogSpec { title: "Renommer".into(), text: Some(format!("Dans /{}", parent_of(&item.path))), names: None, input: Some(item.name.clone()), confirm: "Renommer".into(), submit });
    }

    /// Copie le chemin absolu de l'élément dans le presse-papiers du système.
    pub fn copy_path(self: &Rc<Self>, item: &Entry) {
        use gtk4::prelude::*;
        let full = format!("{}/{}", self.st.borrow().root.trim_end_matches('/'), item.path);
        self.ui.window.clipboard().set_text(&full);
        self.toast("Chemin copié");
    }
}
