//! Veille des règles : un passage toutes les 5 secondes dans un fil de travail, résultats traités sur le fil de l'interface.

use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use std::path::Path;
use std::rc::{Rc, Weak};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

use five_engine::clock::now_seconds;
use five_engine::jobs::registry::Jobs;
use five_engine::root::Root;
use five_engine::rules::context_build::Extras;
use five_engine::rules::run_types::Outcome;
use five_engine::rules::runner::{RulesService, TickReport};
use five_engine::views::favorite_items;
use gtk4::glib;

use super::editor::state::Editor;
use crate::app::shared::App;
use crate::app::work::spawn_work;

/// Le service des règles partagé avec le fil de travail.
pub struct RulesUi {
    pub service: Arc<Mutex<RulesService>>,
    pub busy: Cell<bool>,
    pub editors: RefCell<Vec<Rc<Editor>>>,
}

impl RulesUi {
    /// Crée le service sur la racine, avec les transferts de l'application et `~/.config/five` comme dossier de configuration.
    pub fn new(root: Root, jobs: Arc<Jobs>, home: &Path) -> RulesUi {
        RulesUi {
            service: Arc::new(Mutex::new(RulesService::new(root, jobs, home.join(".config/five")))),
            busy: Cell::new(false),
            editors: RefCell::new(Vec::new()),
        }
    }
}

impl App {
    /// Dossier de configuration des règles.
    pub fn rules_dir(&self) -> std::path::PathBuf {
        self.home.join(".config/five")
    }

    /// Ce que le moteur ignore : favoris GTK et dossiers épinglés (chemins relatifs).
    fn rule_extras(&self) -> Extras {
        let favorites: HashSet<String> = favorite_items(&self.root).into_iter().map(|e| e.path).collect();
        let pinned: HashSet<String> = self.side_places().into_iter().map(|p| p.path).collect();
        Extras { favorites, pinned, recents: HashSet::new() }
    }

    /// Lance la veille : un passage toutes les 5 secondes tant que l'application est ouverte et que les règles sont actives.
    pub fn start_rules_loop(self: &Rc<Self>) {
        let weak: Weak<App> = Rc::downgrade(self);
        glib::timeout_add_local(Duration::from_secs(5), move || {
            let Some(app) = weak.upgrade() else { return glib::ControlFlow::Break };
            app.rules_tick();
            glib::ControlFlow::Continue
        });
    }

    /// Un passage de la veille dans un fil de travail ; ignoré si le précédent n'est pas fini ou si les règles sont coupées.
    pub fn rules_tick(self: &Rc<Self>) {
        if !self.set.borrow().rules_enabled || self.rules.busy.replace(true) {
            return;
        }
        let (service, extras, app) = (self.rules.service.clone(), self.rule_extras(), self.clone());
        spawn_work(
            move || {
                let mut service = service.lock().unwrap_or_else(PoisonError::into_inner);
                service.extras = extras;
                service.tick(now_seconds())
            },
            move |report| app.rules_done(report),
        );
    }

    /// Traite le résultat d'un passage : actions de l'interface, dossier rechargé, problèmes signalés, page des règles rafraîchie.
    pub fn rules_done(self: &Rc<Self>, report: TickReport) {
        self.rules.busy.set(false);
        let results = &report.results;
        results.iter().filter(|r| r.outcome == Outcome::Deferred).for_each(|r| self.apply_deferred(r));
        self.report_rule_problems(results);
        if results.iter().any(|r| r.outcome == Outcome::Done) {
            self.reload();
            self.poll_jobs();
        }
        if !results.is_empty() && self.st.borrow().settings_open && self.st.borrow().tab == "rules" {
            self.render_settings();
        }
    }
}
