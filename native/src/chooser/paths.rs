//! Chemin saisi dans l'en-tête et sélection courante de la liste.

use std::path::PathBuf;
use std::rc::Rc;

use gtk4::prelude::*;

use super::listing::Row;
use super::request::Mode;
use super::window::Chooser;

impl Chooser {
    /// Chemin saisi : un dossier est ouvert ; en enregistrement, un fichier fixe aussi le nom proposé.
    pub fn submit_path(self: &Rc<Self>, text: &str) {
        let expanded = match (text.strip_prefix("~/"), std::env::var_os("HOME")) {
            (Some(rest), Some(home)) => PathBuf::from(home).join(rest),
            _ => PathBuf::from(text.trim()),
        };
        if expanded.is_dir() {
            return self.go(&expanded);
        }
        let parent = expanded.parent().filter(|p| p.is_dir());
        if let (Some(parent), Mode::Save, Some(name)) = (parent, self.request.mode, expanded.file_name()) {
            self.footer.name.set_text(&name.to_string_lossy());
            self.go(parent);
        }
    }

    /// Lignes actuellement sélectionnées dans la liste.
    pub fn selected_rows(&self) -> Vec<Row> {
        let rows = self.rows.borrow();
        self.list.selected_rows().iter().filter_map(|r| rows.get(r.index() as usize).cloned()).collect()
    }

    /// Sélectionne la ligne portant ce nom (essais et captures en ligne de commande).
    pub fn select_name(&self, name: &str) {
        let index = self.rows.borrow().iter().position(|r| r.name == name);
        if let Some(row) = index.and_then(|i| self.list.row_at_index(i as i32)) {
            self.list.select_row(Some(&row));
        }
    }

    /// Selon le mode : en enregistrement, un clic sur un fichier propose son nom ; la validation est remise à jour.
    pub fn on_selection(self: &Rc<Self>) {
        let picked = self.selected_rows();
        self.preview.show(if let [row] = picked.as_slice() { Some(row) } else { None });
        if let (Mode::Save, [row]) = (self.request.mode, picked.as_slice())
            && !row.is_dir
        {
            self.footer.name.set_text(&row.name);
        }
        self.refresh_accept();
    }
}
