//! Barre du bas : affichage des raccourcis et ouverture d'un dossier ou d'un fichier épinglé.

use std::path::{Path, PathBuf};
use std::rc::Rc;

use five_engine::entry::Entry;
use gtk4::prelude::*;
use gtk4::{Button, GestureClick};

use super::menu_place::MenuPlace;
use super::popover::MenuEntry;
use super::shared::App;
use crate::ui::classes::clear;
use crate::ui::icon::icon;
use crate::ui::widgets::{hbox, label};

impl App {
    /// Raccourcis qui existent encore et se trouvent sous la racine.
    fn dock_paths(&self) -> Vec<PathBuf> {
        let paths = self.set.borrow().dock.clone();
        paths.iter().map(PathBuf::from).filter(|full| self.root.inside(full) && full.exists()).collect()
    }

    /// Reconstruit la barre du bas ; un message guide l'utilisateur tant qu'elle est vide.
    pub fn render_dock(self: &Rc<Self>) {
        clear(&self.ui.dock);
        let paths = self.dock_paths();
        if paths.is_empty() {
            self.ui.dock.append(&label("Clic droit sur un dossier ou un fichier › Épingler dans la barre du bas", &["dock-hint"]));
        }
        paths.iter().for_each(|full| self.ui.dock.append(&self.dock_chip(full)));
    }

    /// Raccourci : icône et nom ; clic = ouvrir, clic droit = retirer.
    fn dock_chip(self: &Rc<Self>, full: &Path) -> Button {
        let name = full.file_name().map_or_else(String::new, |n| n.to_string_lossy().into_owned());
        let (button, row) = (Button::new(), hbox(8));
        button.add_css_class("dock-chip");
        row.append(&icon(if full.is_dir() { "folder" } else { "file" }));
        let text = label(&name, &[]);
        text.set_max_width_chars(24);
        text.set_ellipsize(gtk4::pango::EllipsizeMode::End);
        row.append(&text);
        button.set_child(Some(&row));
        button.set_tooltip_text(Some(&full.to_string_lossy()));
        let target = full.to_path_buf();
        crate::app::shared::on_click(&button, self, move |app| app.open_dock_path(&target));
        attach_dock_menu(self, &button, full);
        button
    }

    /// Ouvre un raccourci : un dossier dans FIVE, un fichier avec l'application par défaut.
    pub fn open_dock_path(self: &Rc<Self>, full: &Path) {
        let name = full.file_name().map_or_else(String::new, |n| n.to_string_lossy().into_owned());
        match Entry::describe(&self.root, full, &name) {
            Some(item) => self.open_item(&item),
            None => self.toast("Ce raccourci n'existe plus"),
        }
    }
}

/// Clic droit sur un raccourci : propose de le retirer de la barre.
fn attach_dock_menu(app: &Rc<App>, button: &Button, full: &Path) {
    let click = GestureClick::new();
    click.set_button(3);
    let (weak, path, anchor) = (Rc::downgrade(app), full.to_string_lossy().into_owned(), button.clone());
    click.connect_pressed(move |_, _, x, y| {
        let Some(app) = weak.upgrade() else { return };
        let target = path.clone();
        let entry = MenuEntry::new("x", "Retirer de la barre du bas", move |app| app.unpin_from_dock(&target));
        app.popover_at(&anchor, MenuPlace::Pointer(x, y), vec![entry]);
    });
    button.add_controller(click);
}
