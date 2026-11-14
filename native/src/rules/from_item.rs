//! Créer une règle depuis un fichier ou un dossier : l'éditeur s'ouvre déjà rempli.

use std::rc::Rc;

use five_engine::entry::Entry;
use serde_json::json;

use crate::app::shared::App;
use crate::model::paths::{name_parts, parent_of};
use crate::model::rules_draft::new_rule_draft;

impl App {
    /// Règle qui surveille ce dossier (sans sous-dossiers) ; il reste à choisir les conditions et les actions.
    pub fn rule_from_folder(self: &Rc<Self>, folder: &Entry) {
        let mut draft = new_rule_draft();
        draft["name"] = json!(format!("Surveiller {}", folder.name));
        draft["watch"][0]["path"] = json!(folder.path);
        self.open_rule_draft(draft);
    }

    /// Règle qui surveille le dossier du fichier et vise les fichiers de même extension (ou de même nom, sans extension).
    pub fn rule_from_file(self: &Rc<Self>, file: &Entry) {
        let mut draft = new_rule_draft();
        let (_, dotted) = name_parts(&file.name);
        let ext = dotted.trim_start_matches('.').to_lowercase();
        draft["watch"][0]["path"] = json!(parent_of(&file.path));
        let test = if ext.is_empty() {
            json!({"type": "name", "matcher": {"op": "equals", "value": file.name}})
        } else {
            json!({"type": "extension", "in_list": [ext]})
        };
        draft["name"] = json!(if ext.is_empty() { format!("Règle pour {}", file.name) } else { format!("Règle pour les fichiers .{ext}") });
        draft["alternatives"] = json!([{"conditions": [{"negated": false, "test": test}]}]);
        self.open_rule_draft(draft);
    }
}
