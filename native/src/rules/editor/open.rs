//! Ouverture de l'éditeur : fenêtre, pied de page (annuler, aperçu, enregistrer), enregistrement de la règle.

use std::cell::RefCell;
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{Box as GtkBox, PolicyType, ScrolledWindow};
use serde_json::Value;

use super::catalog::Catalog;
use super::draft::load_draft;
use super::sections::build_body;
use super::state::Editor;
use crate::app::shared::App;
use crate::ui::widgets::{label, vbox};

impl App {
    /// Garde l'éditeur en vie tant que sa fenêtre est ouverte (les boutons ne le référencent que faiblement) ; oublie les éditeurs fermés.
    fn keep_editor(&self, editor: &Rc<Editor>) {
        let mut editors = self.rules.editors.borrow_mut();
        editors.retain(|e| self.modals.borrow().iter().any(|m| m.layer == e.modal.layer));
        editors.push(editor.clone());
    }

    /// Corps défilant de l'éditeur : sa hauteur suit celle de la fenêtre (68 % au plus).
    fn editor_scroll(&self) -> (GtkBox, ScrolledWindow) {
        let body = vbox(14);
        let max = (self.ui.window.height() as f64 * 0.68) as i32;
        let scroll = ScrolledWindow::builder()
            .hscrollbar_policy(PolicyType::Never)
            .min_content_height(300)
            .max_content_height(max.max(300))
            .propagate_natural_height(true)
            .child(&body)
            .build();
        (body, scroll)
    }

    /// Ouvre l'éditeur d'une règle existante (`Some(id)`) ou d'une nouvelle règle (`None`).
    pub fn open_rule_editor(self: &Rc<Self>, id: Option<String>) {
        let draft = load_draft(self, &id);
        self.open_editor(id, draft);
    }

    /// Ouvre l'éditeur sur une règle neuve préremplie (depuis un fichier ou un dossier).
    pub fn open_rule_draft(self: &Rc<Self>, draft: Value) {
        self.open_editor(None, draft);
    }

    /// Fenêtre de l'éditeur sur `draft` ; `id` est celui de la règle enregistrée à remplacer, s'il y en a une.
    fn open_editor(self: &Rc<Self>, id: Option<String>, draft: Value) {
        let modal = self.open_modal(920, false);
        modal.dialog.add_css_class("dialog");
        modal.dialog.append(&label(if id.is_some() { "MODIFIER LA RÈGLE" } else { "NOUVELLE RÈGLE" }, &["h2"]));
        let (body, scroll) = self.editor_scroll();
        let notice = label("", &["setting-text", "rule-reason"]);
        notice.set_wrap(true);
        notice.set_max_width_chars(1);
        let editor = Rc::new(Editor {
            app: Rc::downgrade(self),
            modal: modal.clone(),
            catalog: Catalog::load(),
            draft: RefCell::new(draft),
            id,
            body,
            scroll: scroll.clone(),
            notice: notice.clone(),
        });
        build_body(&editor);
        self.keep_editor(&editor);
        modal.dialog.append(&scroll);
        modal.dialog.append(&notice);
        modal.dialog.append(&editor.footer());
    }
}
