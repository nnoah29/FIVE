//! Presse-papiers de fichiers : copier, couper, coller.

use std::rc::Rc;

use five_engine::entry::Entry;
use gtk4::prelude::*;
use five_engine::jobs::start::start_job;

use super::shared::App;
use super::work::spawn_work;
use crate::state::Clip;
use crate::ui::classes::set_button_text;

impl App {
    /// Mémorise des éléments à copier (`copy`) ou à déplacer (`move`).
    pub fn set_clip(self: &Rc<Self>, mode: &str, items: &[Entry]) {
        if items.is_empty() {
            return self.toast("Sélectionnez au moins un élément");
        }
        self.st.borrow_mut().clip = Some(Clip { mode: mode.to_string(), paths: items.iter().map(|i| i.path.clone()).collect() });
        self.toast(&format!("{} {}", items.len(), if mode == "copy" { "copié(s)" } else { "coupé(s)" }));
        self.update_paste();
    }

    /// Affiche le bouton Coller quand il y a quelque chose à coller dans un dossier ordinaire.
    pub fn update_paste(self: &Rc<Self>) {
        let st = self.st.borrow();
        self.ui.files.paste.set_visible(st.clip.is_some() && st.can_create());
        if let Some(clip) = &st.clip {
            set_button_text(&self.ui.files.paste, &format!("Coller ({})", clip.paths.len()));
        }
    }

    /// Lance le transfert des éléments mémorisés vers le dossier courant.
    pub fn paste_clip(self: &Rc<Self>) {
        let Some(clip) = self.st.borrow().clip.clone() else { return self.toast("Rien à coller : copiez ou coupez d'abord") };
        if !self.st.borrow().can_create() {
            return self.toast("Ouvrez un dossier ordinaire pour y coller");
        }
        let (root, jobs, dest, app) = (self.root.clone(), self.jobs.clone(), self.st.borrow().path.clone(), self.clone());
        let mode = clip.mode.clone();
        spawn_work(move || start_job(&root, &jobs, &clip.mode, &clip.paths, &dest).map_err(|r| r.message), move |result| match result {
            Ok(id) => {
                app.jobs_view.borrow_mut().seen.insert(id, "running".to_string());
                if mode == "move" {
                    app.st.borrow_mut().clip = None;
                }
                app.update_paste();
                app.poll_jobs();
            }
            Err(message) => app.toast(&message),
        });
    }
}
