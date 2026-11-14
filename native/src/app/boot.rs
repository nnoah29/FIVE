//! Démarrage : emplacements, disques, session restaurée, première lecture.

use std::rc::Rc;

use five_engine::listing::list_dir;

use super::data::{read_disks, read_places};
use super::shared::App;
use crate::model::session::load_session;
use crate::model::sorting::SortKey;

impl App {
    /// Emplacement de départ : la session précédente si elle est lisible, sinon la racine.
    fn start_location(&self) -> (String, Option<String>) {
        if !self.set.borrow().restore {
            return (String::new(), None);
        }
        let (path, view) = load_session(&self.home);
        let readable = view.is_some() || path.is_empty() || self.root.resolve(&path).ok().is_some_and(|full| list_dir(&self.root, &full, false).is_ok());
        if readable { (path, view) } else { (String::new(), None) }
    }

    /// Prépare l'état depuis les réglages, lit emplacements et disques, ouvre l'emplacement de départ.
    pub fn start(self: &Rc<Self>) {
        {
            let set = self.set.borrow().clone();
            let mut st = self.st.borrow_mut();
            (st.sort, st.ascending, st.hidden) = (SortKey::from_letter(&set.sort), set.ascending, set.hidden);
            st.places = read_places(&self.root);
            st.disks = read_disks(&self.root);
        }
        self.apply_theme();
        self.sync_toggles();
        self.apply_layout();
        let (path, view) = self.start_location();
        self.load(path, view);
        self.poll_jobs();
        self.watch_wallpaper();
        self.start_rules_loop();
    }
}
