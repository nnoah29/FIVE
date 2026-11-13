//! Application des réglages : thème, couleur, densité, affichage, tri, fichiers cachés.

use std::rc::Rc;

use super::shared::App;
use crate::model::settings::Settings;
use crate::model::sorting::SortKey;
use crate::ui::classes::set_class;
use crate::ui::scheme::is_dark;
use crate::ui::theme::build_css;

impl App {
    /// Régénère la feuille de style selon le thème, la couleur principale, la densité et le format des dates.
    pub fn apply_theme(self: &Rc<Self>) {
        let settings = self.set.borrow().clone();
        self.provider.load_from_string(&build_css(is_dark(&settings), &settings));
    }

    /// Reflète les réglages sur les boutons de la fenêtre (affichage, fichiers cachés, paramètres).
    pub fn sync_toggles(self: &Rc<Self>) {
        let (view, hidden) = (self.set.borrow().view.clone(), self.set.borrow().hidden);
        set_class(&self.ui.files.view_list, "pressed", view == "list");
        set_class(&self.ui.files.view_grid, "pressed", view == "grid");
        set_class(&self.ui.top.hidden, "pressed", hidden);
        set_class(&self.ui.top.gear, "pressed", self.st.borrow().settings_open);
    }

    /// Modifie un réglage, l'enregistre, l'applique et redessine, y compris la page Paramètres ouverte.
    pub fn change_setting(self: &Rc<Self>, change: impl FnOnce(&mut Settings)) {
        self.change_setting_quiet(change);
        if self.st.borrow().settings_open {
            self.render_settings();
        }
    }

    /// Comme `change_setting`, sans redessiner la page Paramètres (saisie en cours conservée).
    pub fn change_setting_quiet(self: &Rc<Self>, change: impl FnOnce(&mut Settings)) {
        let before = self.set.borrow().clone();
        change(&mut self.set.borrow_mut());
        let after = self.set.borrow().clone();
        after.save(&self.home);
        self.apply_theme();
        {
            let mut st = self.st.borrow_mut();
            (st.sort, st.ascending, st.hidden) = (SortKey::from_letter(&after.sort), after.ascending, after.hidden);
        }
        self.sync_toggles();
        if before.hidden != after.hidden {
            return self.reload();
        }
        self.st.borrow_mut().sel = -1;
        self.render(None);
        self.show_preview();
    }
}
