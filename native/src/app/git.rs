//! Indicateurs Git, état restauré après chargement.

use std::rc::Rc;

use five_engine::entry::Entry;
use five_engine::git_marks::git_marks;
use gtk4::prelude::*;
use gtk4::Widget;

use super::shared::App;
use super::work::spawn_work;
use crate::model::session::save_session;
use crate::ui::widgets::label;

/// Libellé d'une marque Git, pour l'info-bulle.
fn git_label(code: char) -> &'static str {
    match code {
        'M' => "Modifié",
        'A' => "Ajouté",
        'D' => "Supprimé",
        '?' => "Non suivi",
        '!' => "Ignoré",
        _ => "Conflit",
    }
}

impl App {
    /// Badge Git d'un élément : lettre colorée, ou widget invisible s'il n'y a rien à dire.
    pub fn git_badge(&self, item: &Entry) -> Widget {
        let st = self.st.borrow();
        let code = if st.git_path.as_deref() == Some(st.path.as_str()) { st.git.get(&item.name).copied() } else { None };
        let Some(code) = code else {
            let none = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
            none.set_visible(false);
            return none.upcast();
        };
        let class = match code {
            '?' => "git-q".to_string(),
            '!' => "git-x".to_string(),
            other => format!("git-{other}"),
        };
        let badge = label(&code.to_string(), &["git", &class]);
        badge.set_tooltip_text(Some(git_label(code)));
        badge.upcast()
    }

    /// Cherche l'état Git du dossier courant en arrière-plan et redessine s'il y a des marques.
    pub fn load_git(self: &Rc<Self>) {
        {
            let mut st = self.st.borrow_mut();
            st.git.clear();
            st.git_path = None;
            if !self.set.borrow().git || st.virtual_view.is_some() || st.searching() {
                return;
            }
        }
        let path = self.st.borrow().path.clone();
        let Ok(full) = self.root.resolve(&path) else { return };
        let app = self.clone();
        spawn_work(move || git_marks(&full), move |marks| {
            if marks.is_empty() || app.st.borrow().path != path {
                return;
            }
            {
                let mut st = app.st.borrow_mut();
                st.git = marks;
                st.git_path = Some(path);
            }
            app.render(None);
        });
    }

    /// Après chaque chargement : sélection différée, session enregistrée, état Git.
    pub fn after_load(self: &Rc<Self>) {
        let pending = self.st.borrow_mut().pending.take();
        if let Some(path) = pending {
            let index = self.st.borrow().shown.iter().position(|i| i.path == path);
            if let Some(index) = index {
                self.select(index as isize, gtk4::gdk::ModifierType::empty());
            }
        }
        {
            let st = self.st.borrow();
            save_session(&self.home, &st.path, st.virtual_view.as_deref());
        }
        self.load_git();
    }
}
