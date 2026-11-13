//! Palette de commandes (Ctrl P / Ctrl K) : commandes et fichiers de la racine, recherche floue.

use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};

use five_engine::fuzzy::fuzzy_score;
use five_engine::search::search_dir;
use gtk4::prelude::*;
use gtk4::{Button, Entry};

use super::modal::Modal;
use super::palette_commands::commands;
use super::palette_parts::{file_row, key_controller, results_area};
use super::shared::App;
use super::work::spawn_work;
use crate::ui::classes::{clear, set_class};
use crate::ui::icon::icon;
use crate::ui::widgets::{hbox, label, mono};

pub type Action = Rc<dyn Fn(&Rc<App>)>;

/// Ligne de résultat : icône, texte, précision, étiquette, action.
pub struct Row {
    pub icon: String,
    pub text: String,
    pub sub: Option<String>,
    pub tag: String,
    pub run: Action,
}

/// État de la palette ouverte.
pub struct Palette {
    pub app: Weak<App>,
    pub modal: Modal,
    pub input: Entry,
    pub results: gtk4::Box,
    pub rows: RefCell<Vec<Row>>,
    pub index: Cell<usize>,
    pub ticket: Cell<u64>,
}

impl Palette {
    /// Dessine les résultats ; `note` ajoute une ligne d'information en bas.
    pub fn draw(self: &Rc<Self>, note: &str) {
        clear(&self.results);
        for (at, row) in self.rows.borrow().iter().enumerate() {
            let button = Button::new();
            button.add_css_class("res");
            set_class(&button, "sel", at == self.index.get());
            let line = hbox(12);
            line.append(&icon(&row.icon));
            let text = hbox(0);
            text.set_hexpand(true);
            text.append(&label(&row.text, &[]));
            if let Some(sub) = &row.sub {
                text.append(&label(sub, &["res-sub"]));
            }
            line.append(&text);
            line.append(&mono(&row.tag, &["res-tag", "dim"]));
            button.set_child(Some(&line));
            let me = self.clone();
            button.connect_clicked(move |_| me.run(at));
            self.results.append(&button);
        }
        if !note.is_empty() {
            self.results.append(&mono(note, &["dim", "res-note"]));
        }
    }

    /// Exécute la ligne `at` et ferme la palette.
    pub fn run(self: &Rc<Self>, at: usize) {
        let action = self.rows.borrow().get(at).map(|r| r.run.clone());
        let Some(app) = self.app.upgrade() else { return };
        app.close_modal(&self.modal);
        if let Some(action) = action {
            action(&app);
        }
    }

    /// Commandes correspondant à la saisie, les huit meilleures.
    fn command_rows(&self, app: &Rc<App>) -> Vec<Row> {
        let query = self.input.text().trim().to_string();
        let mut scored: Vec<(i64, Row)> = commands(app)
            .into_iter()
            .filter_map(|(ic, text, hint, run)| {
                let score = if query.is_empty() { Some(0) } else { fuzzy_score(&query, &text) }?;
                Some((score, Row { icon: ic, text, sub: None, tag: hint.unwrap_or("Commande").into(), run }))
            })
            .collect();
        scored.sort_by_key(|(score, _)| *score);
        scored.into_iter().take(8).map(|(_, row)| row).collect()
    }

    /// Recalcule les commandes, puis cherche des fichiers si la saisie fait deux caractères ou plus.
    fn refresh(self: &Rc<Self>) {
        let Some(app) = self.app.upgrade() else { return };
        *self.rows.borrow_mut() = self.command_rows(&app);
        self.index.set(0);
        let query = self.input.text().trim().to_string();
        if query.chars().count() < 2 {
            return self.draw("");
        }
        self.draw("Recherche dans le dossier personnel…");
        let ticket = self.ticket.get() + 1;
        self.ticket.set(ticket);
        let (root, hidden, me) = (app.root.clone(), app.st.borrow().hidden, self.clone());
        spawn_work(
            move || search_dir(&root, root.path(), &query, hidden, true),
            move |found| {
                let Some(app) = me.app.upgrade().filter(|_| me.ticket.get() == ticket) else { return };
                let mut rows = me.command_rows(&app);
                let empty = found.is_empty();
                rows.extend(found.into_iter().take(20).map(file_row));
                *me.rows.borrow_mut() = rows;
                me.draw(if empty { "Aucun fichier ne correspond." } else { "" });
            },
        );
    }
}

impl App {
    /// Ouvre la palette de commandes.
    pub fn open_palette(self: &Rc<Self>) {
        if self.modal_open() {
            return;
        }
        let modal = self.open_modal(640, true);
        modal.dialog.add_css_class("palette");
        modal.dialog.set_spacing(0);
        let input = crate::app::dialog::field("");
        input.set_placeholder_text(Some("Chercher un fichier ou une commande"));
        let (results, scroll) = results_area(self.ui.window.height());
        modal.dialog.append(&input);
        modal.dialog.append(&scroll);
        let palette = Rc::new(Palette {
            app: Rc::downgrade(self),
            modal,
            input: input.clone(),
            results,
            rows: RefCell::new(Vec::new()),
            index: Cell::new(0),
            ticket: Cell::new(0),
        });
        let typed = palette.clone();
        input.connect_changed(move |_| typed.refresh());
        input.add_controller(key_controller(&palette));
        input.grab_focus();
        palette.refresh();
    }
}
