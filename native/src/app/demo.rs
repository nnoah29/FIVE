//! Pilotage par script pour les captures de vérification : `--shot fichier.png=commande,commande`.

use std::rc::Rc;
use std::time::Duration;

use gtk4::gdk::ModifierType;
use gtk4::glib;
use gtk4::prelude::*;

use super::demo_find::{find_class, find_widget};
use super::shared::App;
use crate::ui::capture::save_png;

impl App {
    /// Commandes de démonstration de navigation et d'affichage ; `false` si la commande est inconnue ici.
    fn demo_nav(self: &Rc<Self>, name: &str, arg: &str) -> bool {
        match name {
            "go" => self.go(arg),
            "goview" => self.go_view(arg),
            "view" => self.set_view(arg),
            "theme" => self.change_setting(|s| s.theme = arg.to_string()),
            "accent" => self.change_setting(|s| s.accent = arg.to_string()),
            "compact" => self.change_setting(|s| s.compact = arg == "1"),
            "filter" => self.toggle_filter(arg),
            "search" => self.ui.top.search.set_text(arg),
            "side" => self.open_side(),
            "size" => {
                let (w, h) = arg.split_once('x').unwrap_or(("1500", "900"));
                self.ui.window.set_default_size(w.parse().unwrap_or(1500), h.parse().unwrap_or(900));
            }
            "tab" => {
                self.st.borrow_mut().tab = arg.to_string();
                self.render_settings();
            }
            _ => return false,
        }
        true
    }

    /// Commandes de démonstration de sélection, de fenêtres et d'actions.
    fn demo_action(self: &Rc<Self>, name: &str, arg: &str) {
        let index: usize = arg.parse().unwrap_or(0);
        match name {
            "select" => self.select(index as isize, ModifierType::empty()),
            "rulepreview" => self.open_rule_preview(arg),
            "zip" => self.archive_items("zip", &self.picked()),
            "unzip" => self.archive_items("unzip", &self.picked()),
            "range" => {
                let (a, b) = arg.split_once('-').unwrap_or(("0", "0"));
                self.select(a.parse().unwrap_or(0), ModifierType::empty());
                self.select(b.parse().unwrap_or(0), ModifierType::SHIFT_MASK);
            }
            "settings" => self.open_settings(),
            "palette" => self.open_palette(),
            "quicklook" => self.quick_look(None),
            "usage" => self.analyse_usage(),
            "dups" => self.find_duplicates(),
            "rename" => self.rename_items(self.picked()),
            "delete" => self.delete_items(self.picked()),
            "menu" => {
                let item = self.st.borrow().shown.get(index).cloned();
                if let (Some(item), Some(row)) = (item, self.row_widgets().get(index).cloned()) {
                    self.open_menu_at(&row, &item, Some(index));
                }
            }
            _ => self.demo_dialog(name, arg),
        }
    }

    /// Commandes de démonstration des dialogues et du presse-papiers.
    fn demo_dialog(self: &Rc<Self>, name: &str, arg: &str) {
        match name {
            "mkdir" => self.create_folder(),
            "input" => {
                if let Some(entry) = self.modals.borrow().last().and_then(|m| find_widget::<gtk4::Entry>(m.dialog.upcast_ref())) {
                    entry.set_text(arg);
                }
            }
            "confirm" => {
                let button = self.modals.borrow().last().and_then(|m| find_class(m.dialog.upcast_ref(), "go"));
                if let Some(button) = button.and_then(|b| b.downcast::<gtk4::Button>().ok()) {
                    button.emit_clicked();
                }
            }
            "copy" => self.set_clip("copy", &self.picked()),
            "cut" => self.set_clip("move", &self.picked()),
            "paste" => self.paste_clip(),
            _ => {}
        }
    }

    /// Exécute une commande de démonstration : navigation, sélection, réglages, fenêtres.
    fn demo_command(self: &Rc<Self>, command: &str) {
        let (name, arg) = command.split_once(':').unwrap_or((command, ""));
        if !self.demo_nav(name, arg) {
            self.demo_action(name, arg);
        }
    }

    /// Joue les captures demandées : une étape toutes les 900 ms (commande, ou enregistrement du PNG).
    pub fn run_shots(self: &Rc<Self>, shots: Vec<(String, Vec<String>)>) {
        let mut steps: Vec<(bool, String)> = Vec::new();
        for (file, commands) in shots {
            steps.extend(commands.into_iter().map(|c| (false, c)));
            steps.push((true, file));
        }
        let (app, mut queue) = (self.clone(), steps.into_iter());
        glib::timeout_add_local(Duration::from_millis(900), move || match queue.next() {
            Some((true, file)) => {
                save_png(&app.ui.window, &file);
                glib::ControlFlow::Continue
            }
            Some((false, command)) => {
                app.demo_command(&command);
                glib::ControlFlow::Continue
            }
            None => {
                if let Some(application) = app.ui.window.application() {
                    application.quit();
                }
                glib::ControlFlow::Break
            }
        });
    }
}
