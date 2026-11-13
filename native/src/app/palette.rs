//! Palette de commandes (Ctrl P / Ctrl K) : commandes et fichiers de la racine, recherche floue.

use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};

use five_engine::fuzzy::fuzzy_score;
use five_engine::search::search_dir;
use gtk4::gdk::Key;
use gtk4::glib::Propagation;
use gtk4::prelude::*;
use gtk4::{Button, Entry, EventControllerKey, PolicyType, ScrolledWindow};

use super::modal::Modal;
use super::shared::App;
use super::work::spawn_work;
use crate::model::kinds::{kind_of, kind_icon};
use crate::model::paths::parent_of;
use crate::ui::classes::{clear, set_class};
use crate::ui::icon::icon;
use crate::ui::widgets::{hbox, label, mono, vbox};

type Action = Rc<dyn Fn(&Rc<App>)>;

/// Ligne de résultat : icône, texte, précision, étiquette, action.
struct Row {
    icon: String,
    text: String,
    sub: Option<String>,
    tag: String,
    run: Action,
}

/// État de la palette ouverte.
struct Palette {
    app: Weak<App>,
    modal: Modal,
    input: Entry,
    results: gtk4::Box,
    rows: RefCell<Vec<Row>>,
    index: Cell<usize>,
    ticket: Cell<u64>,
}

/// Commandes proposées : icône, libellé, raccourci affiché, action.
fn commands(app: &Rc<App>) -> Vec<(String, String, Option<&'static str>, Action)> {
    let act = |f: fn(&Rc<App>)| -> Action { Rc::new(f) };
    let mut list: Vec<(String, String, Option<&'static str>, Action)> = vec![
        ("plus".into(), "Nouveau dossier".into(), Some("c"), act(|a| a.create_folder())),
        ("up".into(), "Dossier parent".into(), Some("h"), act(|a| a.go_up())),
        ("paste".into(), "Coller".into(), Some("Ctrl V"), act(|a| a.paste_clip())),
        ("pen".into(), "Renommer la sélection".into(), Some("F2"), act(|a| a.rename_items(a.st.borrow().picked_items()))),
        ("gear".into(), "Ouvrir les paramètres".into(), Some(","), act(|a| a.toggle_settings())),
        ("sun".into(), "Basculer le thème clair / sombre".into(), None, act(|a| a.toggle_theme())),
        ("eye".into(), "Afficher ou masquer les fichiers cachés".into(), Some("."), act(|a| a.toggle_hidden())),
        ("list".into(), "Affichage en liste".into(), None, act(|a| a.set_view("list"))),
        ("grid".into(), "Affichage en grille".into(), None, act(|a| a.set_view("grid"))),
        ("archive".into(), "Analyser l'occupation de ce dossier".into(), None, act(|a| a.analyse_usage())),
        ("copy".into(), "Chercher les doublons dans ce dossier".into(), None, act(|a| a.find_duplicates())),
        ("home".into(), "Aller à : Dossier personnel".into(), None, act(|a| a.go(""))),
        ("recent".into(), "Aller à : Récents".into(), None, act(|a| a.go_view("recent"))),
        ("favorites".into(), "Aller à : Favoris".into(), None, act(|a| a.go_view("favorites"))),
        ("network".into(), "Aller à : Réseau".into(), None, act(|a| a.go_view("network"))),
    ];
    for place in app.st.borrow().places.clone() {
        let path = place.path.clone();
        list.push((place.id.clone(), format!("Aller à : {}", place.label), None, Rc::new(move |a: &Rc<App>| a.go(&path))));
    }
    list
}

impl Palette {
    /// Dessine les résultats ; `note` ajoute une ligne d'information en bas.
    fn draw(self: &Rc<Self>, note: &str) {
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
    fn run(self: &Rc<Self>, at: usize) {
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
        spawn_work(move || search_dir(&root, root.path(), &query, hidden, true), move |found| {
            let Some(app) = me.app.upgrade().filter(|_| me.ticket.get() == ticket) else { return };
            let mut rows = me.command_rows(&app);
            let empty = found.is_empty();
            rows.extend(found.into_iter().take(20).map(|item| {
                let (dir, path) = (item.is_dir, item.path.clone());
                Row { icon: if dir { "folder".into() } else { kind_icon(kind_of(&item)).into() }, text: item.name.clone(), sub: Some(format!("/{}", parent_of(&item.path))), tag: if dir { "Dossier" } else { "Fichier" }.into(), run: Rc::new(move |a: &Rc<App>| if dir { a.go(&path) } else { a.st.borrow_mut().pending = Some(path.clone()); a.go(&parent_of(&path)) }) }
            }));
            *me.rows.borrow_mut() = rows;
            me.draw(if empty { "Aucun fichier ne correspond." } else { "" });
        });
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
        let results = vbox(0);
        results.add_css_class("results");
        let scroll = ScrolledWindow::builder().hscrollbar_policy(PolicyType::Never).max_content_height((self.ui.window.height() as f64 * 0.5) as i32).propagate_natural_height(true).child(&results).build();
        modal.dialog.append(&input);
        modal.dialog.append(&scroll);
        let palette = Rc::new(Palette { app: Rc::downgrade(self), modal, input: input.clone(), results, rows: RefCell::new(Vec::new()), index: Cell::new(0), ticket: Cell::new(0) });
        let typed = palette.clone();
        input.connect_changed(move |_| typed.refresh());
        let keys = EventControllerKey::new();
        keys.set_propagation_phase(gtk4::PropagationPhase::Capture);
        let nav = palette.clone();
        keys.connect_key_pressed(move |_, key, _, _| {
            let count = nav.rows.borrow().len();
            match key {
                Key::Down | Key::Up if count > 0 => {
                    let next = (nav.index.get() as isize + if key == Key::Down { 1 } else { -1 }).clamp(0, count as isize - 1);
                    nav.index.set(next as usize);
                    nav.draw("");
                    Propagation::Stop
                }
                Key::Return | Key::KP_Enter => {
                    nav.run(nav.index.get());
                    Propagation::Stop
                }
                _ => Propagation::Proceed,
            }
        });
        input.add_controller(keys);
        input.grab_focus();
        palette.refresh();
    }
}
