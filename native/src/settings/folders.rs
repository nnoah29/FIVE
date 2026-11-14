//! Onglet « Dossiers » : choisir les dossiers de la barre latérale, leur nom et leur ordre.

use std::path::Path;
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Entry, Widget, gio};

use super::controls::{choice, setting_row};
use super::panels::opt;
use crate::app::shared::{App, on_click};
use crate::model::side_folders::{SideFolder, move_folder, shown_label};
use crate::ui::buttons::icon_button;
use crate::ui::icon::icon;
use crate::ui::widgets::{hbox, label, text_button};

impl App {
    /// Ligne d'un dossier épinglé : icône, nom modifiable, chemin, monter, descendre, retirer.
    fn folder_row(self: &Rc<Self>, at: usize, folder: &SideFolder, last: usize) -> GtkBox {
        let row = hbox(12);
        row.add_css_class("setting");
        row.append(&icon(&folder.icon));
        let entry = Entry::new();
        entry.add_css_class("hex");
        entry.set_text(&folder.label);
        entry.set_placeholder_text(Some(&shown_label(folder)));
        entry.set_hexpand(true);
        let weak = Rc::downgrade(self);
        entry.connect_changed(move |e| {
            let (text, app) = (e.text().to_string(), weak.upgrade());
            let edit =
                move |s: &mut crate::model::settings::Settings| s.side_folders.as_mut().and_then(|l| l.get_mut(at)).map(|f| f.label = text).unwrap_or_default();
            if let Some(app) = app {
                app.change_setting_quiet(edit);
            }
        });
        row.append(&entry);
        let missing = if Path::new(&folder.path).is_dir() { "" } else { " · introuvable" };
        let path = label(&format!("{}{missing}", folder.path), &["mono", "dim"]);
        path.set_ellipsize(gtk4::pango::EllipsizeMode::Start);
        path.set_max_width_chars(34);
        path.set_tooltip_text(Some(&folder.path));
        row.append(&path);
        self.folder_buttons(&row, at, last);
        row
    }

    /// Boutons d'une ligne : monter, descendre, retirer.
    fn folder_buttons(self: &Rc<Self>, row: &GtkBox, at: usize, last: usize) {
        for (glyph, delta) in [("up", -1isize), ("down", 1)] {
            let button = icon_button(glyph, &["small"]);
            button.set_sensitive(if delta < 0 { at > 0 } else { at < last });
            on_click(&button, self, move |app| app.edit_side_folders(|list| move_folder(list, at, delta)));
            row.append(&button);
        }
        let remove = icon_button("x", &["small"]);
        remove.set_tooltip_text(Some("Retirer de la barre latérale"));
        on_click(&remove, self, move |app| app.edit_side_folders(|list| drop(list.remove(at))));
        row.append(&remove);
    }

    /// Ouvre le sélecteur de dossier système ; le dossier choisi est épinglé s'il est sous la racine.
    fn choose_folder(self: &Rc<Self>) {
        let dialog = gtk4::FileDialog::new();
        dialog.set_initial_folder(Some(&gio::File::for_path(self.root.path())));
        let app = self.clone();
        dialog.select_folder(Some(&self.ui.window), None::<&gio::Cancellable>, move |result| {
            let Some(chosen) = result.ok().and_then(|file| file.path()) else { return };
            match chosen.strip_prefix(app.root.path()).map(Path::to_path_buf) {
                Ok(rel) => app.pin_side_folder(&rel.to_string_lossy()),
                Err(_) => app.toast("Ce dossier est hors de la racine exposée"),
            }
        });
    }

    /// Ligne des deux boutons d'ajout : choisir un dossier, ajouter le dossier ouvert.
    fn folder_actions(self: &Rc<Self>) -> GtkBox {
        let row = hbox(12);
        row.add_css_class("setting");
        let pick = text_button("Choisir un dossier…", Some("plus"), &[]);
        on_click(&pick, self, |app| app.choose_folder());
        let here = text_button("Ajouter le dossier ouvert", Some("folder"), &[]);
        on_click(&here, self, |app| {
            let path = app.st.borrow().path.clone();
            if path.is_empty() || app.st.borrow().virtual_view.is_some() {
                app.toast("Ouvrez d'abord un dossier à épingler")
            } else {
                app.pin_side_folder(&path)
            }
        });
        row.set_halign(Align::Fill);
        row.append(&pick);
        row.append(&here);
        row
    }

    /// Onglet « Dossiers » : mode automatique ou personnalisé, liste modifiable, ajout.
    pub fn folders_panel(self: &Rc<Self>) -> Vec<Widget> {
        let list = self.set.borrow().side_folders.clone();
        let defaults = self.default_side_folders();
        let mode = choice(
            self,
            vec![
                opt("Automatique", list.is_none(), |s| s.side_folders = None),
                opt("Personnalisés", list.is_some(), move |s| {
                    s.side_folders.get_or_insert_with(|| defaults.clone());
                }),
            ],
        );
        let text = "Automatique : vos dossiers habituels (Téléchargements, Images, Musique, Vidéos…). Personnalisés : épinglez les vôtres, renommez-les et changez leur ordre.";
        let mut rows: Vec<Widget> = vec![setting_row("Dossiers de la barre latérale", Some(text), &mode, false).upcast()];
        if let Some(list) = list {
            let last = list.len().saturating_sub(1);
            rows.extend(list.iter().enumerate().map(|(at, f)| self.folder_row(at, f, last).upcast::<Widget>()));
            rows.push(self.folder_actions().upcast());
        }
        rows
    }
}
