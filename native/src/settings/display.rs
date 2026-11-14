//! Panneau « Affichage » de la page Paramètres : thème, couleurs, densité, disposition, unités, dates.

use std::rc::Rc;

use gtk4::Widget;
use gtk4::prelude::*;

use super::controls::{checkbox, choice, setting_row, toggle_row};
use super::panels::opt;
use crate::app::shared::App;
use crate::model::settings::Settings;

impl App {
    /// Choix du thème : système, clair ou sombre.
    fn theme_choice(self: &Rc<Self>, s: &Settings) -> gtk4::Box {
        choice(
            self,
            vec![
                opt("Système", s.theme == "system", |s| s.theme = "system".into()),
                opt("Clair", s.theme == "light", |s| s.theme = "light".into()),
                opt("Sombre", s.theme == "dark", |s| s.theme = "dark".into()),
            ],
        )
    }

    /// Lignes de mise en page : densité, disposition, miniatures.
    fn layout_rows(self: &Rc<Self>, s: &Settings) -> Vec<Widget> {
        let density = choice(self, vec![opt("Confortable", !s.compact, |s| s.compact = false), opt("Compacte", s.compact, |s| s.compact = true)]);
        let view = choice(self, vec![opt("Liste", s.view == "list", |s| s.view = "list".into()), opt("Grille", s.view == "grid", |s| s.view = "grid".into())]);
        let thumbs = checkbox(self, s.thumbs, |s, v| s.thumbs = v);
        vec![
            setting_row("Densité de la liste", Some("Confortable : lignes aérées. Compacte : plus de fichiers à l'écran."), &density, false).upcast(),
            setting_row("Affichage des fichiers", Some("Disposition utilisée à chaque ouverture."), &view, false).upcast(),
            setting_row("Miniatures des images", Some("En grille, affiche l'image plutôt qu'une icône."), &thumbs, false).upcast(),
        ]
    }

    /// Lignes de format : unité des tailles, format des dates.
    fn format_rows(self: &Rc<Self>, s: &Settings) -> Vec<Widget> {
        let units = choice(self, vec![opt("Ko", !s.binary_units, |s| s.binary_units = false), opt("Kio", s.binary_units, |s| s.binary_units = true)]);
        let dates = choice(self, vec![opt("Courte", !s.full_dates, |s| s.full_dates = false), opt("Complète", s.full_dates, |s| s.full_dates = true)]);
        vec![
            setting_row("Unité des tailles", Some("1 Ko = 1 000 octets, ou 1 Kio = 1 024 octets."), &units, false).upcast(),
            setting_row("Format des dates", Some("Courte : l'heure pour aujourd'hui, sinon la date."), &dates, false).upcast(),
        ]
    }

    /// Panneau « Affichage » : thème, fond d'écran, couleur, densité, disposition, miniatures, unités, dates.
    pub fn display_panel(self: &Rc<Self>) -> Vec<Widget> {
        let s = self.set.borrow().clone();
        let follow = "Reprend les couleurs de votre fond d'écran, comme MIRA ; sans effet si le bureau ne les fournit pas.";
        let mut rows: Vec<Widget> = vec![
            setting_row("Thème", Some("« Système » suit le réglage de votre ordinateur."), &self.theme_choice(&s), false).upcast(),
            toggle_row(self, "Suivre le fond d'écran", follow, s.wallpaper, |s, v| s.wallpaper = v).upcast(),
            setting_row("Couleur principale", Some("Remplace l'orange : boutons, accents, sélection et libellés actifs."), &self.accent_control(), true)
                .upcast(),
        ];
        rows.extend(self.layout_rows(&s));
        rows.extend(self.format_rows(&s));
        rows
    }
}
