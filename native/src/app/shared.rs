//! Structure centrale de l'application, partagée par tous les gestionnaires d'événements.

use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;

use five_engine::jobs::registry::Jobs;
use five_engine::root::Root;
use gtk4::{CssProvider, Popover};

use crate::model::settings::Settings;
use crate::state::State;
use crate::ui::handles::Ui;

use super::modal::Modal;

/// Suivi des transferts affichés.
#[derive(Default)]
pub struct JobsView {
    pub seen: HashMap<String, String>,
    pub hidden: HashSet<String>,
    pub fading: HashSet<String>,
    pub started: bool,
    pub poll_gen: u64,
}

/// L'application : widgets, état, réglages, moteur.
pub struct App {
    pub ui: Ui,
    pub st: RefCell<State>,
    pub set: RefCell<Settings>,
    pub root: Root,
    pub jobs: Arc<Jobs>,
    pub provider: CssProvider,
    pub home: PathBuf,
    pub toast_gen: Cell<u64>,
    pub search_gen: Cell<u64>,
    pub compact: Cell<bool>,
    pub modals: RefCell<Vec<Modal>>,
    pub menu: RefCell<Option<Popover>>,
    pub jobs_view: RefCell<JobsView>,
}

/// Référence partagée vers l'application, clonée dans les gestionnaires.
pub type AppRef = Rc<App>;

/// Branche une action sur un bouton ; l'application n'est retenue que faiblement.
pub fn on_click(button: &gtk4::Button, app: &AppRef, action: impl Fn(&AppRef) + 'static) {
    use gtk4::prelude::ButtonExt;
    let weak = Rc::downgrade(app);
    button.connect_clicked(move |_| {
        if let Some(app) = weak.upgrade() {
            action(&app);
        }
    });
}
