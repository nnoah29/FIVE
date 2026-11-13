//! Panneaux de la page Paramètres : affichage, fichiers, raccourcis, système.

use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{Align, Grid, Widget};

use super::controls::{checkbox, choice, setting_row, Choice};
use crate::app::shared::App;
use crate::model::shortcuts::SHORTCUTS;
use crate::preview::facts::Facts;
use crate::ui::widgets::{hbox, label, vbox};

/// Option d'un groupe : libellé, état courant, modification.
fn opt(text: &'static str, pressed: bool, change: impl Fn(&mut crate::model::settings::Settings) + 'static) -> Choice {
    (text, pressed, Rc::new(change))
}

impl App {
    /// Panneau « Affichage » : thème, couleur, densité, disposition, miniatures, unités, dates.
    pub fn display_panel(self: &Rc<Self>) -> Vec<Widget> {
        let s = self.set.borrow().clone();
        let theme = choice(self, vec![opt("Système", s.theme == "system", |s| s.theme = "system".into()), opt("Clair", s.theme == "light", |s| s.theme = "light".into()), opt("Sombre", s.theme == "dark", |s| s.theme = "dark".into())]);
        let density = choice(self, vec![opt("Confortable", !s.compact, |s| s.compact = false), opt("Compacte", s.compact, |s| s.compact = true)]);
        let view = choice(self, vec![opt("Liste", s.view == "list", |s| s.view = "list".into()), opt("Grille", s.view == "grid", |s| s.view = "grid".into())]);
        let units = choice(self, vec![opt("Ko", !s.binary_units, |s| s.binary_units = false), opt("Kio", s.binary_units, |s| s.binary_units = true)]);
        let dates = choice(self, vec![opt("Courte", !s.full_dates, |s| s.full_dates = false), opt("Complète", s.full_dates, |s| s.full_dates = true)]);
        let thumbs = checkbox(self, s.thumbs, |s, v| s.thumbs = v);
        vec![
            setting_row("Thème", Some("« Système » suit le réglage de votre ordinateur."), &theme, false).upcast(),
            setting_row("Couleur principale", Some("Remplace l'orange : boutons, accents, sélection et libellés actifs."), &self.accent_control(), true).upcast(),
            setting_row("Densité de la liste", Some("Confortable : lignes aérées. Compacte : plus de fichiers à l'écran."), &density, false).upcast(),
            setting_row("Affichage des fichiers", Some("Disposition utilisée à chaque ouverture."), &view, false).upcast(),
            setting_row("Miniatures des images", Some("En grille, affiche l'image plutôt qu'une icône."), &thumbs, false).upcast(),
            setting_row("Unité des tailles", Some("1 Ko = 1 000 octets, ou 1 Kio = 1 024 octets."), &units, false).upcast(),
            setting_row("Format des dates", Some("Courte : l'heure pour aujourd'hui, sinon la date."), &dates, false).upcast(),
        ]
    }

    /// Panneau « Fichiers » : cachés, tri, Git, reprise de session, confirmation de suppression.
    pub fn files_panel(self: &Rc<Self>) -> Vec<Widget> {
        let s = self.set.borrow().clone();
        let sort = choice(self, vec![opt("Nom", s.sort == "n", |s| s.sort = "n".into()), opt("Type", s.sort == "t", |s| s.sort = "t".into()), opt("Taille", s.sort == "s", |s| s.sort = "s".into()), opt("Date", s.sort == "m", |s| s.sort = "m".into())]);
        let order = choice(self, vec![opt("Croissant", s.ascending, |s| s.ascending = true), opt("Décroissant", !s.ascending, |s| s.ascending = false)]);
        vec![
            setting_row("Afficher les fichiers cachés", Some("Ceux dont le nom commence par un point."), &checkbox(self, s.hidden, |s, v| s.hidden = v), false).upcast(),
            setting_row("Tri par défaut", Some("Les dossiers restent toujours avant les fichiers."), &sort, false).upcast(),
            setting_row("Ordre du tri", None, &order, false).upcast(),
            setting_row("Indicateurs Git", Some("Affiche M, ?, A, D ou ! à côté des fichiers d'un dépôt Git."), &checkbox(self, s.git, |s, v| s.git = v), false).upcast(),
            setting_row("Reprendre où je me suis arrêté", Some("Rouvre le dernier dossier consulté au démarrage."), &checkbox(self, s.restore, |s, v| s.restore = v), false).upcast(),
            setting_row("Confirmer la mise à la corbeille", Some("La suppression définitive demande toujours une confirmation."), &checkbox(self, s.confirm_trash, |s, v| s.confirm_trash = v), false).upcast(),
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
        vec![facts.grid.clone().upcast(), setting_row("Changer de racine", Some("Relancez FIVE avec --root /chemin (par exemple --root / pour tout le disque)."), &note, false).upcast()]
    }
}
