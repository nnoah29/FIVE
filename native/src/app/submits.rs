//! Validations des dialogues : création de dossier, suppression, renommage (travail en arrière-plan).

use std::rc::Rc;

use five_engine::delete::delete_entry;
use five_engine::folders::make_folder;
use five_engine::rename::rename_many;
use five_engine::root::Root;

use super::dialog::Submit;
use super::work::spawn_work;

/// Met chaque chemin à la corbeille (ou l'efface s'il y est déjà), en s'arrêtant à la première erreur.
fn delete_all(root: &Root, paths: &[String]) -> Result<(), String> {
    paths.iter().try_for_each(|p| delete_entry(root, p).map(|_| ()).map_err(|r| r.message))
}

/// Validation du dialogue « Nouveau dossier ».
pub fn mkdir_submit() -> Submit {
    Rc::new(|app, name, report| {
        let (root, parent, app) = (app.root.clone(), app.st.borrow().path.clone(), app.clone());
        spawn_work(
            move || make_folder(&root, &parent, &name.unwrap_or_default()).map(|_| ()).map_err(|r| r.message),
            move |result| {
                if result.is_ok() {
                    app.toast("Dossier créé");
                    app.reload();
                }
                report(result);
            },
        );
    })
}

/// Validation de la suppression des chemins donnés ; `definitive` règle le message.
pub fn delete_submit(paths: Vec<String>, definitive: bool) -> Submit {
    Rc::new(move |app, _, report| {
        let (root, list, app) = (app.root.clone(), paths.clone(), app.clone());
        spawn_work(
            move || delete_all(&root, &list),
            move |result| {
                if result.is_ok() {
                    app.toast(if definitive { "Supprimé définitivement" } else { "Mis à la corbeille" });
                    app.st.borrow_mut().sel = -1;
                    app.reload();
                }
                report(result);
            },
        );
    })
}

/// Validation du renommage d'un élément ; le nouveau nom vient de la saisie.
pub fn rename_submit(path: String) -> Submit {
    Rc::new(move |app, name, report| {
        let (root, pair, app) = (app.root.clone(), (path.clone(), name.unwrap_or_default()), app.clone());
        spawn_work(
            move || rename_many(&root, &[pair]).map(|_| ()).map_err(|r| r.message),
            move |result| {
                if result.is_ok() {
                    app.toast("Renommé");
                    app.reload();
                }
                report(result);
            },
        );
    })
}
