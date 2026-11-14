//! État de l'éditeur : brouillon JSON, modification par chemin, reconstruction de la fenêtre.

use std::cell::RefCell;
use std::rc::{Rc, Weak};

use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Label, ScrolledWindow};
use serde_json::Value;

use super::catalog::Catalog;
use super::sections::build_body;
use crate::app::modal::Modal;
use crate::app::shared::App;
use crate::model::rules_draft::{Seg, at_mut};
use crate::ui::classes::clear;

/// Un éditeur ouvert : la règle en cours d'édition et les widgets à reconstruire.
pub struct Editor {
    pub app: Weak<App>,
    pub modal: Modal,
    pub catalog: Catalog,
    pub id: Option<String>,
    pub draft: RefCell<Value>,
    pub body: GtkBox,
    pub scroll: ScrolledWindow,
    pub notice: Label,
}

impl Editor {
    /// L'application, si elle existe encore.
    pub fn app(&self) -> Option<Rc<App>> {
        self.app.upgrade()
    }

    /// Fixe la valeur au bout du chemin (sans reconstruire la fenêtre : la saisie en cours est conservée).
    pub fn set(&self, path: &[Seg], value: Value) {
        if let Some(slot) = at_mut(&mut self.draft.borrow_mut(), path) {
            *slot = value;
        }
    }

    /// Valeur au bout du chemin (copie).
    pub fn get(&self, path: &[Seg]) -> Value {
        at_mut(&mut self.draft.borrow_mut(), path).map_or(Value::Null, |v| v.clone())
    }

    /// Modifie la structure du brouillon puis reconstruit la fenêtre en gardant la position de défilement.
    pub fn mutate(self: &Rc<Self>, change: impl FnOnce(&mut Value)) {
        change(&mut self.draft.borrow_mut());
        let position = self.scroll.vadjustment().value();
        self.modal.layer.set_focusable(true);
        self.modal.layer.grab_focus();
        clear(&self.body);
        build_body(self);
        let (scroll, layer) = (self.scroll.clone(), self.modal.layer.clone());
        glib::idle_add_local_once(move || {
            scroll.vadjustment().set_value(position);
            layer.grab_focus();
        });
    }
}
