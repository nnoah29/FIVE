//! Actions de règles que seule l'interface peut faire : notification, favoris, épinglage, ouverture.

use std::fs;
use std::rc::Rc;

use five_engine::entry::Entry;
use five_engine::rules::run_types::{ActionResult, Outcome};

use crate::app::shared::App;
use crate::model::bookmarks::toggle_bookmark;
use crate::model::paths::{leaf, parent_of};

impl App {
    /// Ajoute ou retire un dossier des favoris GTK (fichier `bookmarks`).
    fn edit_favorites(&self, rel: &str, add: bool) {
        let Ok(full) = self.root.resolve_entry(rel) else { return };
        let file = self.home.join(".config/gtk-3.0/bookmarks");
        let text = fs::read_to_string(&file).unwrap_or_default();
        let updated = toggle_bookmark(&text, &full.to_string_lossy(), add);
        if let Some(parent) = file.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::write(file, updated);
    }

    /// Exécute une action confiée à l'interface par le moteur.
    pub fn apply_deferred(self: &Rc<Self>, result: &ActionResult) {
        match result.action.as_str() {
            "notify" | "notify_only_once" => self.toast(&result.detail),
            "add_to_favorites" => self.edit_favorites(&result.path, true),
            "remove_from_favorites" => self.edit_favorites(&result.path, false),
            "pin_folder_in_sidebar" => self.pin_side_folder(&result.path),
            "open_with_default" => {
                let described = self.root.resolve_entry(&result.path).ok().and_then(|full| Entry::describe(&self.root, &full, &leaf(&result.path)));
                described.iter().for_each(|entry| self.open_item(entry));
            }
            "reveal_in_five_folder" => self.go(&parent_of(&result.path)),
            _ => {}
        }
    }

    /// Signale le premier échec et la première question d'une passe (une seule alerte par passe).
    pub fn report_rule_problems(self: &Rc<Self>, results: &[ActionResult]) {
        if let Some(failed) = results.iter().find(|r| r.outcome == Outcome::Failed) {
            self.toast(&format!("Règle : échec — {}", failed.detail));
        } else if let Some(ask) = results.iter().find(|r| r.outcome == Outcome::Ask) {
            self.toast(&format!("Règle : « {} » attend votre choix", leaf(&ask.path)));
        }
    }
}
