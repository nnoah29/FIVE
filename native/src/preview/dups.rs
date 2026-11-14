//! Doublons : groupes de fichiers identiques, cases à cocher, mise à la corbeille des copies.

use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;
use std::time::Duration;

use five_engine::delete::delete_entry;
use five_engine::duplicates::find_duplicates;
use gtk4::prelude::*;
use gtk4::{Align, Button, Label, PolicyType, ScrolledWindow};
use serde_json::Value;

use crate::app::modal::Modal;
use crate::app::shared::{App, on_click};
use crate::app::work::spawn_work;
use crate::model::format::format_size;
use crate::ui::classes::set_button_text;
use crate::ui::controls::is_ticked;
use crate::ui::widgets::{hbox, label, text_button, vbox};

/// Ensemble des copies cochées (le premier fichier de chaque groupe est conservé).
pub type Marked = Rc<RefCell<HashSet<String>>>;

/// Fichiers cochés au départ : toutes les copies sauf la première de chaque groupe.
fn initial_marks(data: &Value) -> HashSet<String> {
    let groups = data["groups"].as_array().cloned().unwrap_or_default();
    groups.iter().flat_map(|g| g["files"].as_array().cloned().unwrap_or_default().into_iter().skip(1).filter_map(|f| f.as_str().map(String::from))).collect()
}

impl App {
    /// Ouvre la fenêtre des doublons et lance la recherche dans le dossier courant.
    pub fn find_duplicates(self: &Rc<Self>) {
        if self.st.borrow().virtual_view.is_some() {
            return;
        }
        let path = self.st.borrow().path.clone();
        let modal = self.open_modal(720, false);
        modal.dialog.add_css_class("dialog");
        modal.dialog.append(&label(&format!("DOUBLONS DANS /{path}"), &["h2"]));
        modal.dialog.append(&label("Analyse en cours : comparaison des tailles puis des empreintes SHA-256…", &["dialog-text"]));
        let Ok(full) = self.root.resolve(&path) else { return };
        let (root, app) = (self.root.clone(), self.clone());
        spawn_work(move || find_duplicates(&root, &full, 1024, Duration::from_secs(20)), move |data| app.show_duplicates(&modal, data));
    }

    /// Remplit la fenêtre avec les résultats : résumé, groupes, boutons.
    fn show_duplicates(self: &Rc<Self>, modal: &Modal, data: Value) {
        let box_ = &modal.dialog;
        while let Some(child) = box_.first_child() {
            box_.remove(&child);
        }
        box_.append(&label("DOUBLONS", &["h2"]));
        let summary: Label = label("", &["dialog-text"]);
        summary.set_wrap(true);
        summary.set_max_width_chars(1);
        box_.append(&summary);
        let groups = data["groups"].as_array().cloned().unwrap_or_default();
        let marked: Marked = Rc::new(RefCell::new(initial_marks(&data)));
        let confirm: Button = text_button("Mettre à la corbeille", Some("trash2"), &["go"]);
        let refresh = self.dup_refresh(&data, &marked, &summary, &confirm);
        if !groups.is_empty() {
            let list = vbox(14);
            groups.iter().for_each(|g| list.append(&self.dup_group(g, &marked, &refresh)));
            box_.append(
                &ScrolledWindow::builder().hscrollbar_policy(PolicyType::Never).max_content_height(380).propagate_natural_height(true).child(&list).build(),
            );
        }
        box_.append(&self.dup_actions(modal, &confirm, !groups.is_empty()));
        self.dup_wire_confirm(&confirm, &marked, modal);
        refresh();
    }

    /// Rangée de boutons : Fermer, et la confirmation s'il y a des doublons.
    fn dup_actions(self: &Rc<Self>, modal: &Modal, confirm: &Button, any: bool) -> gtk4::Box {
        let actions = hbox(8);
        actions.set_halign(Align::End);
        let close = text_button("Fermer", None, &[]);
        let target = modal.clone();
        on_click(&close, self, move |app| app.close_modal(&target));
        actions.append(&close);
        if any {
            actions.append(confirm);
        }
        actions
    }

    /// Fonction qui réécrit le résumé et le bouton selon les cases cochées.
    fn dup_refresh(self: &Rc<Self>, data: &Value, marked: &Marked, summary: &Label, confirm: &Button) -> Rc<dyn Fn()> {
        let (data, marked, summary, confirm, units) = (data.clone(), marked.clone(), summary.clone(), confirm.clone(), self.set.borrow().binary_units);
        Rc::new(move || {
            let groups = data["groups"].as_array().cloned().unwrap_or_default();
            let freed: u64 = groups
                .iter()
                .map(|g| {
                    g["size"].as_u64().unwrap_or(0)
                        * g["files"].as_array().map_or(0, |f| f.iter().filter(|x| x.as_str().is_some_and(|s| marked.borrow().contains(s))).count()) as u64
                })
                .sum();
            let partial = if data["partial"].as_bool().unwrap_or(false) { " Analyse partielle." } else { "" };
            summary.set_text(&if groups.is_empty() {
                format!("Aucun doublon trouvé dans ce dossier.{partial}")
            } else {
                format!(
                    "{} groupe(s). Gaspillage : {}. Le premier fichier de chaque groupe est conservé.{partial}",
                    groups.len(),
                    format_size(data["wasted"].as_u64().unwrap_or(0), units)
                )
            });
            set_button_text(&confirm, &format!("Mettre {} fichier(s) à la corbeille ({})", marked.borrow().len(), format_size(freed, units)));
            confirm.set_sensitive(!marked.borrow().is_empty());
        })
    }

    /// Bouton de confirmation : met les copies cochées à la corbeille.
    fn dup_wire_confirm(self: &Rc<Self>, confirm: &Button, marked: &Marked, modal: &Modal) {
        let (marked, modal) = (marked.clone(), modal.clone());
        on_click(confirm, self, move |app| {
            let (paths, root, done, modal) = (marked.borrow().iter().cloned().collect::<Vec<_>>(), app.root.clone(), app.clone(), modal.clone());
            spawn_work(
                move || paths.iter().try_for_each(|p| delete_entry(&root, p).map(|_| ())).map_err(|r| r.message),
                move |result| match result {
                    Ok(()) => {
                        done.toast("Mis à la corbeille");
                        done.close_modal(&modal);
                        done.reload();
                    }
                    Err(message) => done.toast(&message),
                },
            );
        });
        let _ = is_ticked;
    }
}
