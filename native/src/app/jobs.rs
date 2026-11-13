//! File de transferts : suivi des copies et déplacements, pause, reprise, annulation.

use std::rc::Rc;
use std::time::Duration;

use five_engine::jobs::start::control_job;
use gtk4::glib;
use serde_json::Value;

use super::jobs_view::is_live;
use super::shared::App;

impl App {
    /// Relit les transferts, recharge le dossier quand l'un se termine, redessine, replanifie tant qu'un transfert est actif.
    pub fn poll_jobs(self: &Rc<Self>) {
        let list = self.jobs.view();
        let mut finished = false;
        let mut fresh: Vec<String> = Vec::new();
        {
            let mut view = self.jobs_view.borrow_mut();
            if !view.started {
                view.started = true;
                let old: Vec<String> = list.iter().filter(|j| !is_live(j)).filter_map(|j| j["id"].as_str().map(String::from)).collect();
                view.hidden.extend(old);
            }
            for job in &list {
                let (id, state) = (job["id"].as_str().unwrap_or("").to_string(), job["state"].as_str().unwrap_or("").to_string());
                let before = view.seen.insert(id.clone(), state.clone());
                finished |= before.is_some_and(|b| b != state) && !is_live(job);
                if !is_live(job) && view.fading.insert(id.clone()) {
                    fresh.push(id);
                }
            }
        }
        fresh.into_iter().for_each(|id| self.hide_job_later(id));
        if finished {
            self.reload();
        }
        let shown: Vec<Value> = list.iter().filter(|j| !self.jobs_view.borrow().hidden.contains(j["id"].as_str().unwrap_or(""))).cloned().collect();
        self.draw_jobs(&shown);
        if list.iter().any(is_live) {
            self.schedule_poll();
        }
    }

    /// Masque un transfert terminé au bout de 6 secondes.
    fn hide_job_later(self: &Rc<Self>, id: String) {
        let weak = Rc::downgrade(self);
        glib::timeout_add_local_once(Duration::from_secs(6), move || {
            if let Some(app) = weak.upgrade() {
                app.jobs_view.borrow_mut().hidden.insert(id);
                app.poll_jobs();
            }
        });
    }

    /// Replanifie une lecture dans 500 ms ; une lecture plus récente annule la précédente.
    fn schedule_poll(self: &Rc<Self>) {
        let generation = {
            let mut view = self.jobs_view.borrow_mut();
            view.poll_gen += 1;
            view.poll_gen
        };
        let weak = Rc::downgrade(self);
        glib::timeout_add_local_once(Duration::from_millis(500), move || {
            if let Some(app) = weak.upgrade().filter(|a| a.jobs_view.borrow().poll_gen == generation) {
                app.poll_jobs();
            }
        });
    }

    /// Met un transfert en pause, le reprend ou l'annule.
    pub fn job_act(self: &Rc<Self>, id: &str, action: &str) {
        if let Err(refus) = control_job(&self.jobs, id, action) {
            self.toast(&refus.message);
        }
        self.poll_jobs();
    }
}
