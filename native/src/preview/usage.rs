//! Analyse d'occupation : barres proportionnelles par enfant du dossier.

use std::rc::Rc;
use std::time::Duration;

use five_engine::usage::usage_of;
use gtk4::prelude::*;
use gtk4::{Align, Button, DrawingArea, Overlay};

use crate::app::shared::{on_click, App};
use crate::app::work::spawn_work;
use crate::model::format::format_size;
use crate::state::UsageView;
use crate::ui::icon::icon;
use crate::ui::widgets::{hbox, label, text_button, vbox};

/// Barre fine en pied de ligne, de largeur proportionnelle à `ratio`.
fn bar(ratio: f64) -> DrawingArea {
    let area = DrawingArea::new();
    area.set_content_height(2);
    area.set_valign(Align::End);
    area.set_can_target(false);
    area.add_css_class("use-bar-area");
    area.set_draw_func(move |widget, cr, width, height| {
        let color = widget.color();
        cr.set_source_rgba(color.red() as f64, color.green() as f64, color.blue() as f64, 1.0);
        cr.rectangle(0.0, 0.0, width as f64 * ratio, height as f64);
        let _ = cr.fill();
    });
    area
}

impl App {
    /// Lance l'analyse du dossier courant en arrière-plan.
    pub fn analyse_usage(self: &Rc<Self>) {
        if self.st.borrow().virtual_view.is_some() {
            return;
        }
        let path = self.st.borrow().path.clone();
        self.st.borrow_mut().usage = Some(UsageView { path: path.clone(), data: None });
        self.show_preview();
        let Ok(full) = self.root.resolve(&path) else { return };
        let (root, app) = (self.root.clone(), self.clone());
        spawn_work(move || usage_of(&root, &full, Duration::from_secs(10)).map_err(|e| five_engine::refus::Refus::from(e).message), move |result| {
            match result {
                Ok(data) => app.st.borrow_mut().usage = Some(UsageView { path: path.clone(), data: Some(data) }),
                Err(message) => {
                    app.st.borrow_mut().usage = None;
                    app.toast(&message);
                }
            }
            if app.st.borrow().path == path {
                app.show_preview();
            }
        });
    }

    /// Ligne d'un élément : icône, nom, taille, barre proportionnelle ; cliquable si c'est un dossier.
    fn use_row(self: &Rc<Self>, row: &serde_json::Value, max: u64) -> Overlay {
        let (name, is_dir, size) = (row["n"].as_str().unwrap_or("").to_string(), row["d"].as_bool().unwrap_or(false), row["s"].as_u64().unwrap_or(0));
        let button = Button::new();
        button.add_css_class("use");
        button.set_sensitive(is_dir);
        let line = hbox(10);
        line.append(&icon(if is_dir { "folder" } else { "file" }));
        let title = label(&name, &["use-name"]);
        title.set_hexpand(true);
        title.set_ellipsize(gtk4::pango::EllipsizeMode::End);
        line.append(&title);
        line.append(&label(&format_size(size, self.set.borrow().binary_units), &["kind-size"]));
        button.set_child(Some(&line));
        let target = row["p"].as_str().unwrap_or("").to_string();
        on_click(&button, self, move |app| app.go(&target));
        let overlay = Overlay::new();
        overlay.set_child(Some(&button));
        overlay.add_overlay(&bar((size as f64 / max.max(1) as f64).max(0.02)));
        overlay
    }

    /// Bloc d'occupation : en cours, ou liste des 14 plus gros éléments avec retour aux catégories.
    pub fn usage_block(self: &Rc<Self>) -> gtk4::Box {
        let block = vbox(0);
        let data = self.st.borrow().usage.as_ref().and_then(|u| u.data.clone());
        let Some(data) = data else {
            block.append(&label("ANALYSE EN COURS…", &["kinds-title"]));
            return block;
        };
        let units = self.set.borrow().binary_units;
        let partial = if data["partial"].as_bool().unwrap_or(false) { " (partielle)" } else { "" };
        block.append(&label(&format!("OCCUPATION : {}{partial}", format_size(data["total"].as_u64().unwrap_or(0), units)).to_uppercase(), &["kinds-title"]));
        let rows = data["items"].as_array().cloned().unwrap_or_default();
        let max = rows.first().and_then(|r| r["s"].as_u64()).unwrap_or(1);
        rows.iter().take(14).for_each(|row| block.append(&self.use_row(row, max)));
        let back = text_button("Revenir aux catégories", None, &[]);
        back.set_margin_top(12);
        on_click(&back, self, |app| {
            app.st.borrow_mut().usage = None;
            app.show_preview();
        });
        block.append(&back);
        block
    }
}
