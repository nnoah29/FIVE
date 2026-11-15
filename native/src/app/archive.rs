//! Compresser en zip et extraire une archive : lancés comme des transferts, dans le dossier du premier élément.

use std::rc::Rc;

use five_engine::entry::Entry;
use five_engine::jobs::start::start_job;
use gtk4::Button;

use super::popover::MenuEntry;
use super::shared::{App, on_click};
use super::work::spawn_work;
use crate::model::paths::{in_trash, parent_of};
use crate::preview::media::is_archive;
use crate::ui::widgets::text_button;

/// Indique si la sélection est une seule archive que FIVE sait extraire.
fn extractable(items: &[Entry]) -> bool {
    matches!(items, [only] if !only.is_dir && is_archive(&only.name))
}

impl App {
    /// Lance `zip` (une archive pour tous les éléments) ou `unzip` (un dossier par archive) à côté du premier élément.
    pub fn archive_items(self: &Rc<Self>, kind: &'static str, items: &[Entry]) {
        let Some(first) = items.first() else { return self.toast("Sélectionnez au moins un élément") };
        if in_trash(&first.path) {
            return self.toast("Sortez d'abord l'élément de la corbeille");
        }
        let (root, jobs, app) = (self.root.clone(), self.jobs.clone(), self.clone());
        let (paths, dest) = (items.iter().map(|i| i.path.clone()).collect::<Vec<_>>(), parent_of(&first.path));
        spawn_work(
            move || start_job(&root, &jobs, kind, &paths, &dest).map_err(|r| r.message),
            move |result| match result {
                Ok(id) => {
                    app.jobs_view.borrow_mut().seen.insert(id, "running".to_string());
                    app.poll_jobs();
                }
                Err(message) => app.toast(&message),
            },
        );
    }

    /// Bouton du panneau : « Extraire ici » pour une archive seule, rien sinon ni dans la corbeille.
    pub fn archive_button(self: &Rc<Self>, items: &[Entry]) -> Option<Button> {
        if items.is_empty() || !extractable(items) || items.iter().any(|i| in_trash(&i.path)) {
            return None;
        }
        let button = text_button("Extraire ici", Some("archive"), &[]);
        let list = items.to_vec();
        on_click(&button, self, move |app| app.archive_items("unzip", &list));
        Some(button)
    }

    /// Entrées du menu contextuel : compresser la sélection, et extraire si c'est une archive.
    pub fn archive_entries(&self, targets: &[Entry]) -> Vec<MenuEntry> {
        if targets.iter().any(|t| in_trash(&t.path)) {
            return Vec::new();
        }
        let (zip, unzip) = (targets.to_vec(), targets.to_vec());
        let mut out = vec![MenuEntry::new("archive", "Compresser en .zip", move |app| app.archive_items("zip", &zip))];
        if extractable(targets) {
            out.push(MenuEntry::new("archive", "Extraire ici", move |app| app.archive_items("unzip", &unzip)));
        }
        out
    }
}
