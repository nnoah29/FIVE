//! Lancement du sélecteur en ligne de commande : `five-gtk --chooser open|folder|save` ou `--portal`.

use std::cell::Cell;
use std::path::PathBuf;
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{Application, gio, glib};

use super::request::{Mode, Request};
use super::window::Chooser;

/// Lit les options du sélecteur : `--title`, `--name`, `--folder`, `--multiple`, `--shot`.
fn read_request(mode: Mode, args: &[String]) -> (Request, Option<String>) {
    let (mut request, mut shot) = (Request::new(mode, ""), None);
    let mut it = args.iter();
    while let Some(flag) = it.next() {
        match (flag.as_str(), it.clone().next()) {
            ("--multiple", _) => request.multiple = true,
            ("--title", Some(v)) => request.title.clone_from(v),
            ("--name", Some(v)) => request.current_name.clone_from(v),
            ("--folder", Some(v)) => request.start_dir = PathBuf::from(v),
            ("--shot", Some(v)) => shot = Some(v.clone()),
            _ => continue,
        }
        if flag != "--multiple" {
            it.next();
        }
    }
    (request, shot)
}

/// Mode demandé après `--chooser`.
fn read_mode(word: Option<&String>) -> Mode {
    match word.map(String::as_str) {
        Some("folder") => Mode::Folder,
        Some("save") => Mode::Save,
        _ => Mode::Open,
    }
}

/// Affiche le sélecteur seul : les chemins choisis sont écrits sur la sortie standard, un par ligne.
fn run_chooser(request: Request, shot: Option<String>) -> glib::ExitCode {
    let application = Application::builder().application_id("dev.five.files.chooser").flags(gio::ApplicationFlags::NON_UNIQUE).build();
    let code = Rc::new(Cell::new(1u8));
    let result = code.clone();
    application.connect_activate(move |app| {
        std::mem::forget(app.hold());
        let (quit, result) = (app.clone(), result.clone());
        let chooser = Chooser::show(
            request.clone(),
            Box::new(move |choice| {
                choice.iter().flat_map(|c| c.paths.iter()).for_each(|p| println!("{}", p.display()));
                result.set(u8::from(choice.is_none()));
                quit.quit();
            }),
        );
        super::shot::schedule(&chooser, shot.clone());
        std::mem::forget(chooser);
    });
    application.run_with_args::<&str>(&[]);
    glib::ExitCode::from(code.get())
}

/// Si la ligne de commande demande le sélecteur ou le service du portail, le lance ; sinon `None`.
pub fn run_if_requested() -> Option<glib::ExitCode> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("--chooser") => {
            let (request, shot) = read_request(read_mode(args.get(1)), &args[2.min(args.len())..]);
            Some(run_chooser(request, shot))
        }
        Some("--portal") => Some(super::portal::run()),
        _ => None,
    }
}
