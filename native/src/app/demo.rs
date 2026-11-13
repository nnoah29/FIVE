//! Pilotage par script pour les captures de vérification : `--shot fichier.png=commande,commande`.

use std::rc::Rc;
use std::time::Duration;

use gtk4::gdk::ModifierType;
use gtk4::glib;
use gtk4::prelude::*;

use super::shared::App;
use crate::ui::capture::save_png;

impl App {
    /// Exécute une commande de démonstration : navigation, sélection, réglages, fenêtres.
    fn demo_command(self: &Rc<Self>, command: &str) {
        let (name, arg) = command.split_once(':').unwrap_or((command, ""));
        match name {
            "go" => self.go(arg),
            "view" => self.set_view(arg),
            "theme" => self.change_setting(|s| s.theme = arg.to_string()),
            "accent" => self.change_setting(|s| s.accent = arg.to_string()),
            "compact" => self.change_setting(|s| s.compact = arg == "1"),
            "select" => self.select(arg.parse().unwrap_or(0), ModifierType::empty()),
            "range" => {
                let (a, b) = arg.split_once('-').unwrap_or(("0", "0"));
                self.select(a.parse().unwrap_or(0), ModifierType::empty());
                self.select(b.parse().unwrap_or(0), ModifierType::SHIFT_MASK);
            }
            "settings" => self.open_settings(),
            "tab" => {
                self.st.borrow_mut().tab = arg.to_string();
                self.render_settings();
            }
            "palette" => self.open_palette(),
            "quicklook" => self.quick_look(None),
            "usage" => self.analyse_usage(),
            "dups" => self.find_duplicates(),
            "rename" => self.rename_items(self.st.borrow().picked_items()),
            "mkdir" => self.create_folder(),
            "delete" => self.delete_items(self.st.borrow().picked_items()),
            "filter" => self.toggle_filter(arg),
            "menu" => {
                let item = self.st.borrow().shown.get(arg.parse().unwrap_or(0)).cloned();
                if let (Some(item), Some(row)) = (item, self.row_widgets().get(arg.parse().unwrap_or(0)).cloned()) {
                    self.open_menu_at(&row, &item, Some(arg.parse().unwrap_or(0)));
                }
            }
            "dbg" => {
                let mut cursor = self.ui.folders.cards.first_child();
                while let Some(card) = cursor {
                    let mut child = card.first_child();
                    while let Some(c) = child {
                        eprintln!("CARD child {:?} {}x{} classes={:?}", c.type_().name(), c.width(), c.height(), c.css_classes());
                        child = c.next_sibling();
                    }
                    cursor = card.next_sibling();
                }
            }
            "search" => self.ui.top.search.set_text(arg),
            _ => {}
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
