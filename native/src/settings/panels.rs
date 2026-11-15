//! Panneaux de la page Paramètres : affichage, fichiers, raccourcis, système.

use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Grid, Widget};

use super::controls::{Choice, choice, setting_row, toggle_row};
use crate::app::shared::App;
use crate::model::settings::Settings;
use crate::model::shortcuts::SHORTCUTS;
use crate::preview::facts::Facts;
use crate::ui::widgets::{hbox, label, vbox};

/// Option d'un groupe : libellé, état courant, modification.
pub(super) fn opt(text: &'static str, pressed: bool, change: impl Fn(&mut crate::model::settings::Settings) + 'static) -> Choice {
    (text, pressed, Rc::new(change))
}

impl App {
    /// Choix de la colonne et du sens du tri par défaut.
    fn sort_controls(self: &Rc<Self>, s: &Settings) -> (GtkBox, GtkBox) {
        let sort = choice(
            self,
            vec![
                opt("Nom", s.sort == "n", |s| s.sort = "n".into()),
                opt("Type", s.sort == "t", |s| s.sort = "t".into()),
                opt("Taille", s.sort == "s", |s| s.sort = "s".into()),
                opt("Date", s.sort == "m", |s| s.sort = "m".into()),
            ],
        );
        let order = choice(self, vec![opt("Croissant", s.ascending, |s| s.ascending = true), opt("Décroissant", !s.ascending, |s| s.ascending = false)]);
        (sort, order)
    }

    /// Panneau « Fichiers » : cachés, tri, Git, reprise de session, confirmation de suppression.
    pub fn files_panel(self: &Rc<Self>) -> Vec<Widget> {
        let s = self.set.borrow().clone();
        let (sort, order) = self.sort_controls(&s);
        vec![
            toggle_row(self, "Afficher les fichiers cachés", "Ceux dont le nom commence par un point.", s.hidden, |s, v| s.hidden = v),
            setting_row("Tri par défaut", Some("Les dossiers restent toujours avant les fichiers."), &sort, false).upcast(),
            setting_row("Ordre du tri", None, &order, false).upcast(),
            toggle_row(self, "Indicateurs Git", "Affiche M, ?, A, D ou ! à côté des fichiers d'un dépôt Git.", s.git, |s, v| s.git = v),
            toggle_row(self, "Reprendre où je me suis arrêté", "Rouvre le dernier dossier consulté au démarrage.", s.restore, |s, v| s.restore = v),
            toggle_row(self, "Confirmer la mise à la corbeille", "La suppression définitive demande toujours une confirmation.", s.confirm_trash, |s, v| {
                s.confirm_trash = v
            }),
        ]
    }

    /// Panneau « Raccourcis » : touches et effets.
    pub fn keys_panel(self: &Rc<Self>) -> Vec<Widget> {
        let grid = Grid::new();
        for (row, (keys, text)) in SHORTCUTS.iter().enumerate() {
            let caps = hbox(6);
            caps.add_css_class("keys-k");
            keys.split("  ").for_each(|k| caps.append(&label(k, &["kbd"])));
            let meaning = label(text, &["keys-v"]);
            meaning.set_hexpand(true);
            grid.attach(&caps, 0, row as i32, 1, 1);
            grid.attach(&meaning, 1, row as i32, 1, 1);
        }
        vec![grid.upcast()]
    }

    /// Panneau « Système » : racine exposée, corbeille, moteur, changement de racine.
    pub fn system_panel(self: &Rc<Self>) -> Vec<Widget> {
        let facts = Facts::new();
        facts.grid.remove_css_class("sheet");
        facts.add("Racine exposée", &self.st.borrow().root);
        facts.add("Accès", "Cette machine uniquement. Lecture, création de dossiers et mise à la corbeille, dans la racine.");
        facts.add("Corbeille", &format!("/{}", five_engine::places::TRASH));
        facts.add("Moteur", &format!("Natif · five-engine {}", env!("CARGO_PKG_VERSION")));
        let note = vbox(0);
        note.set_halign(Align::End);
        vec![
            facts.grid.clone().upcast(),
            self.chooser_row(),
            setting_row("Changer de racine", Some("Relancez FIVE avec --root /chemin (par exemple --root / pour tout le disque)."), &note, false).upcast(),
        ]
    }
}
