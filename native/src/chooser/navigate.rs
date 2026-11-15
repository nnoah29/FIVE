//! Navigation du sélecteur : aller dans un dossier, monter, recharger, activer une ligne.

use std::path::{Path, PathBuf};
use std::rc::Rc;

use gtk4::prelude::*;

use super::listing::{Row, read_rows, selectable};
use super::request::{Filter, Mode};
use super::rows_view::fill;
use super::window::Chooser;
use crate::ui::classes::set_class;

impl Chooser {
    /// Filtre actif, d'après le menu du pied.
    pub fn active_filter(&self) -> Option<&Filter> {
        let rank = self.footer.filter.as_ref().map(|menu| menu.selected() as usize)?;
        self.request.filters.get(rank)
    }

    /// Affiche le contenu du dossier courant selon le mode, le filtre et les fichiers cachés.
    pub fn reload(self: &Rc<Self>) {
        let dir = self.dir.borrow().clone();
        let rows = read_rows(&dir, self.show_hidden.get()).unwrap_or_default();
        let rows = selectable(rows, self.request.mode, self.active_filter());
        fill(&self.list, &rows);
        *self.rows.borrow_mut() = rows;
        self.preview.show(None);
        self.refresh_accept();
    }

    /// Entre dans un dossier ; ignoré si ce n'en est pas un.
    pub fn go(self: &Rc<Self>, dir: &Path) {
        if !dir.is_dir() {
            return;
        }
        *self.dir.borrow_mut() = dir.to_path_buf();
        self.header.path.set_text(&dir.to_string_lossy());
        self.header.up.set_sensitive(dir.parent().is_some());
        self.reload();
    }

    /// Monte au dossier parent.
    pub fn go_up(self: &Rc<Self>) {
        let parent = self.dir.borrow().parent().map(PathBuf::from);
        parent.iter().for_each(|p| self.go(p));
    }

    /// Affiche ou masque les fichiers cachés.
    pub fn toggle_hidden(self: &Rc<Self>) {
        self.show_hidden.set(!self.show_hidden.get());
        set_class(&self.header.hidden, "pressed", self.show_hidden.get());
        self.reload();
    }

    /// Double-clic ou Entrée sur une ligne : entre dans un dossier, valide un fichier.
    pub fn activate_row(self: &Rc<Self>, index: usize) {
        let row: Option<Row> = self.rows.borrow().get(index).cloned();
        match row {
            Some(row) if row.is_dir && self.request.mode != Mode::Folder => self.go(&row.path),
            Some(_) => self.accept(),
            None => {}
        }
    }
}
