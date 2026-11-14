//! Cartes de la file de transferts : titre, boutons, état, progression.

use std::rc::Rc;

use gtk4::prelude::*;
use serde_json::Value;

use super::job_title::job_title;
use super::shared::{App, on_click};
use crate::ui::buttons::icon_button;
use crate::ui::widgets::{hbox, label, vbox};

/// Libellé français de l'état d'un transfert.
fn state_label(state: &str) -> &'static str {
    match state {
        "running" => "En cours",
        "paused" => "En pause",
        "done" => "Terminé",
        "cancelled" => "Annulé",
        _ => "Erreur",
    }
}

/// Indique si le transfert est encore actif.
pub fn is_live(job: &Value) -> bool {
    matches!(job["state"].as_str(), Some("running" | "paused"))
}

impl App {
    /// Boutons d'une carte : pause ou reprise et annulation, ou masquage si le transfert est fini.
    fn job_buttons(self: &Rc<Self>, top: &gtk4::Box, job: &Value, id: &str, state: &str) {
        let buttons =
            if is_live(job) { vec![if state == "paused" { ("play", "resume") } else { ("pause", "pause") }, ("x", "cancel")] } else { vec![("x", "hide")] };
        for (glyph, action) in buttons {
            let button = icon_button(glyph, &["small"]);
            let target = id.to_string();
            on_click(&button, self, move |app| if action == "hide" { app.hide_job(&target) } else { app.job_act(&target, action) });
            top.append(&button);
        }
    }

    /// Carte d'un transfert : titre, boutons, état, barre de progression.
    fn job_card(self: &Rc<Self>, job: &Value) -> gtk4::Box {
        let (id, state) = (job["id"].as_str().unwrap_or("").to_string(), job["state"].as_str().unwrap_or(""));
        let card = vbox(8);
        card.add_css_class("job");
        let top = hbox(8);
        let title = label(&job_title(job), &["job-title"]);
        title.set_hexpand(true);
        top.append(&title);
        self.job_buttons(&top, job, &id, state);
        let status = label(
            &format!("{} · {}/{} fichiers · vers /{}", state_label(state), job["filesDone"], job["files"], job["dest"].as_str().unwrap_or("")).to_uppercase(),
            &["mono", "dim"],
        );
        let bar = gtk4::ProgressBar::new();
        let total = job["total"].as_f64().unwrap_or(0.0);
        bar.set_fraction(if total > 0.0 {
            (job["done"].as_f64().unwrap_or(0.0) / total).min(1.0)
        } else if state == "done" {
            1.0
        } else {
            0.0
        });
        card.append(&top);
        card.append(&status);
        card.append(&bar);
        card
    }

    /// Masque une carte de transfert terminée.
    pub fn hide_job(self: &Rc<Self>, id: &str) {
        self.jobs_view.borrow_mut().hidden.insert(id.to_string());
        self.poll_jobs();
    }

    /// Redessine la zone des transferts.
    pub fn draw_jobs(self: &Rc<Self>, jobs: &[Value]) {
        crate::ui::classes::clear(&self.ui.transfers);
        self.ui.transfers.set_visible(!jobs.is_empty());
        jobs.iter().for_each(|job| self.ui.transfers.append(&self.job_card(job)));
    }
}
