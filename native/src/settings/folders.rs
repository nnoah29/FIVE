//! Onglet « Dossiers » : choisir les dossiers de la barre latérale, leur nom et leur ordre.

use std::path::Path;
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Widget, gio};

use super::controls::{choice, setting_row};
use super::panels::opt;
use crate::app::shared::{App, on_click};
use crate::model::side_folders::{SideFolder, move_folder, pin_folder};
use crate::ui::buttons::icon_button;
use crate::ui::icon::icon;
use crate::ui::widgets::{hbox, text_button};

impl App {
    /// Ligne d'un dossier épinglé : icône, nom et chemin modifiables, parcourir, monter, descendre, retirer.
    fn folder_row(self: &Rc<Self>, at: usize, folder: &SideFolder, last: usize) -> GtkBox {
        let row = hbox(12);
        row.add_css_class("setting");
        row.append(&icon(&folder.icon));
        row.append(&self.name_entry(at, folder));
        row.append(&self.path_entry(at, folder));
        self.folder_buttons(&row, at, last, &folder.path);
        row
    }

    /// Boutons d'une ligne : parcourir, monter, descendre, retirer.
    fn folder_buttons(self: &Rc<Self>, row: &GtkBox, at: usize, last: usize, path: &str) {
        let browse = icon_button("folder", &["small"]);
        browse.set_tooltip_text(Some("Choisir un autre dossier"));
        let start = path.to_string();
        on_click(&browse, self, move |app| app.choose_folder(&start, move |app, text| app.edit_side_folders(|list| list[at].path = text)));
        row.append(&browse);
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

    /// Ouvre le sélecteur de dossier système, à partir de `start` ; `then` reçoit le chemin choisi s'il est sous la racine.
    pub fn choose_folder(self: &Rc<Self>, start: &str, then: impl Fn(&Rc<App>, String) + 'static) {
        let dialog = gtk4::FileDialog::new();
        let initial = if Path::new(start).is_dir() { Path::new(start) } else { self.root.path() };
        dialog.set_initial_folder(Some(&gio::File::for_path(initial)));
        let app = self.clone();
        dialog.select_folder(Some(&self.ui.window), None::<&gio::Cancellable>, move |result| {
            let Some(chosen) = result.ok().and_then(|file| file.path()) else { return };
            if app.root.inside(&chosen) { then(&app, chosen.to_string_lossy().into_owned()) } else { app.toast("Ce dossier est hors de la racine exposée") }
        });
    }

    /// Ligne des deux boutons d'ajout : choisir un dossier, ajouter le dossier ouvert.
    fn folder_actions(self: &Rc<Self>) -> GtkBox {
        let row = hbox(12);
        row.add_css_class("setting");
        let pick = text_button("Choisir un dossier…", Some("plus"), &[]);
        on_click(&pick, self, |app| {
            app.choose_folder("", |app, text| {
                app.edit_side_folders(|list| {
                    pin_folder(list, &text);
                })
            })
        });
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
