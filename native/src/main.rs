//! FIVE, interface native GTK.

use std::cell::{Cell, RefCell};
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;

use five_engine::jobs::registry::Jobs;
use five_engine::root::Root;
use gtk4::prelude::*;
use gtk4::{Application, CssProvider, gdk, glib};

use five_gtk::app::shared::{App, JobsView};
use five_gtk::model::settings::Settings;
use five_gtk::state::State;
use five_gtk::ui::{build::shell::build_ui, fonts};

/// Options de la ligne de commande.
struct Options {
    root: Option<String>,
    shots: Vec<(String, Vec<String>)>,
}

/// Lit `--root DOSSIER` et les `--shot fichier.png=commande,commande`.
fn options() -> Options {
    let mut args = std::env::args().skip(1);
    let mut out = Options { root: None, shots: Vec::new() };
    while let Some(flag) = args.next() {
        match (flag.as_str(), args.next()) {
            ("--root", Some(value)) => out.root = Some(value),
            ("--shot", Some(value)) => {
                let (file, commands) = value.split_once('=').unwrap_or((&value, ""));
                out.shots.push((file.to_string(), commands.split(',').filter(|c| !c.is_empty()).map(String::from).collect()));
            }
            _ => {}
        }
    }
    out
}

/// Construit l'application : fenêtre, moteur, état, réglages, style.
fn build_app(application: &Application, options: &Options) -> Option<Rc<App>> {
    let home = PathBuf::from(std::env::var("HOME").ok()?);
    let root_path = options.root.as_ref().map(PathBuf::from).unwrap_or_else(|| home.clone());
    let root = Root::new(&root_path, &home).ok()?;
    let ui = build_ui(application);
    let provider = CssProvider::new();
    if let Some(display) = gdk::Display::default() {
        gtk4::style_context_add_provider_for_display(&display, &provider, gtk4::STYLE_PROVIDER_PRIORITY_USER);
    }
    fonts::register(&ui.window);
    let user = std::env::var("USER").unwrap_or_default();
    let state = State::new(root.path().to_string_lossy().into_owned(), user);
    Some(Rc::new(App {
        ui,
        st: RefCell::new(state),
        set: RefCell::new(Settings::load(&home)),
        root,
        jobs: Arc::new(Jobs::new()),
        provider,
        home,
        toast_gen: Cell::new(0),
        search_gen: Cell::new(0),
        compact: Cell::new(false),
        modals: RefCell::new(Vec::new()),
        menu: RefCell::new(None),
        jobs_view: RefCell::new(JobsView::default()),
    }))
}

/// Point d'entrée.
fn main() -> glib::ExitCode {
    five_engine::clock::init_offset();
    let options = Rc::new(options());
    let application = Application::builder().application_id("dev.five.files").flags(gtk4::gio::ApplicationFlags::NON_UNIQUE).build();
    application.connect_activate(move |application| {
        let Some(app) = build_app(application, &options) else { return };
        app.wire();
        app.wire_keys();
        app.ui.window.present();
        app.start();
        let watcher = app.clone();
        let last = Cell::new(0);
        app.ui.window.add_tick_callback(move |window, _| {
            if window.width() != last.get() {
                last.set(window.width());
                watcher.apply_layout();
            }
            glib::ControlFlow::Continue
        });
        if !options.shots.is_empty() {
            if let Some(settings) = gtk4::Settings::default() {
                settings.set_gtk_enable_animations(false);
            }
            app.run_shots(options.shots.clone());
        }
    });
    application.run_with_args::<&str>(&[])
}
