//! Fenêtre du sélecteur : structure, assemblage des zones et affichage.

use std::cell::{Cell, RefCell};
use std::path::PathBuf;
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{ListBox, PolicyType, ScrolledWindow, SelectionMode, Window, WindowHandle};

use super::footer::{Footer, build_footer};
use super::header::{Header, build_header};
use super::listing::Row;
use super::outcome::Choice;
use super::preview::{Preview, build_preview};
use super::request::{Mode, Request};
use super::style;
use crate::ui::widgets::{hbox, vbox};

/// Ce qui est appelé à la fin : le choix de l'utilisateur, ou `None` s'il annule.
pub type Done = Box<dyn FnOnce(Option<Choice>)>;

/// Le sélecteur : demande, widgets et état courant.
pub struct Chooser {
    pub request: Request,
    pub window: Window,
    pub header: Header,
    pub footer: Footer,
    pub list: ListBox,
    pub side: gtk4::Box,
    pub preview: Preview,
    pub dir: RefCell<PathBuf>,
    pub rows: RefCell<Vec<Row>>,
    pub show_hidden: Cell<bool>,
    pub done: RefCell<Option<Done>>,
}

/// Titre proposé quand l'application n'en donne pas.
fn default_title(mode: Mode) -> &'static str {
    match mode {
        Mode::Open => "Ouvrir un fichier",
        Mode::Folder => "Choisir un dossier",
        Mode::Save => "Enregistrer sous",
    }
}

/// Liste des fichiers : sélection multiple seulement si la demande l'autorise.
fn build_list(request: &Request) -> ListBox {
    let list = ListBox::new();
    list.add_css_class("chooser-list");
    let multiple = request.multiple && request.mode != Mode::Save;
    list.set_selection_mode(if multiple { SelectionMode::Multiple } else { SelectionMode::Single });
    list.set_activate_on_single_click(false);
    list
}

/// Corps de la fenêtre : barre latérale à gauche, liste défilante à droite.
fn build_body(list: &ListBox, side: &gtk4::Box, preview: &Preview) -> gtk4::Box {
    let scroll = ScrolledWindow::builder().hscrollbar_policy(PolicyType::Never).child(list).build();
    scroll.set_hexpand(true);
    scroll.set_vexpand(true);
    let side_scroll = ScrolledWindow::builder().hscrollbar_policy(PolicyType::Never).child(side).build();
    side_scroll.set_size_request(210, -1);
    side_scroll.add_css_class("chooser-side");
    let body = hbox(0);
    body.append(&side_scroll);
    body.append(&scroll);
    body.append(&preview.root);
    body
}

/// Construit la fenêtre et son contenu, sans rien câbler.
fn assemble(request: Request, done: Done) -> Rc<Chooser> {
    let title = if request.title.is_empty() { default_title(request.mode) } else { request.title.as_str() };
    let window = Window::builder().title(title).default_width(1040).default_height(580).build();
    window.add_css_class("five");
    window.add_css_class("chooser");
    window.set_titlebar(Some(&gtk4::Box::new(gtk4::Orientation::Vertical, 0)));
    let (header, footer, list, side) = (build_header(title), build_footer(&request), build_list(&request), vbox(2));
    let preview = build_preview();
    let column = vbox(0);
    column.append(&WindowHandle::builder().child(&header.root).build());
    column.append(&build_body(&list, &side, &preview));
    column.append(&footer.root);
    window.set_child(Some(&column));
    let dir = RefCell::new(request.start_dir.clone());
    Rc::new(Chooser {
        request,
        window,
        header,
        footer,
        list,
        side,
        preview,
        dir,
        rows: RefCell::new(Vec::new()),
        show_hidden: Cell::new(false),
        done: RefCell::new(Some(done)),
    })
}

impl Chooser {
    /// Ouvre le sélecteur ; `done` est appelé une seule fois avec le choix ou `None`.
    pub fn show(request: Request, done: Done) -> Rc<Chooser> {
        let chooser = assemble(request, done);
        style::apply(&chooser.window);
        chooser.fill_sidebar();
        chooser.wire();
        let start = chooser.request.start_dir.clone();
        chooser.go(&start);
        chooser.window.present();
        let focus = if chooser.request.mode == Mode::Save { chooser.footer.name.clone().upcast::<gtk4::Widget>() } else { chooser.list.clone().upcast() };
        focus.grab_focus();
        chooser
    }
}
