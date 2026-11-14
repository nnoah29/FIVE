//! Fiche d'un élément sélectionné : titre, aperçu, renseignements, actions.

use std::rc::Rc;

use five_engine::entry::Entry;
use gtk4::Widget;
use gtk4::gdk::ModifierType;
use gtk4::prelude::*;

use super::facts::Facts;
use crate::app::shared::{App, on_click};
use crate::model::format::{format_day, format_size, format_time, group_digits};
use crate::model::kinds::{extension, kind_of};
use crate::model::paths::in_trash;
use crate::ui::widgets::{hbox, label, text_button, vbox};

impl App {
    /// Renseignements de base d'un élément.
    fn item_facts(&self, item: &Entry) -> Rc<Facts> {
        let facts = Facts::new();
        let units = self.set.borrow().binary_units;
        facts.add("Emplacement", &format!("/{}", item.path));
        let kind = if item.is_dir {
            "Dossier".to_string()
        } else {
            item.mime
                .clone()
                .filter(|m| !m.is_empty())
                .or_else(|| Some(extension(&item.name).to_uppercase()).filter(|e| !e.is_empty()))
                .unwrap_or_else(|| "Fichier".into())
        };
        facts.add("Type", &kind);
        if !item.is_dir {
            facts.add("Taille", &format!("{} ({} octets)", format_size(item.size, units), group_digits(item.size)));
        }
        facts.add("Modifié", &format!("{} à {}", format_day(item.modified), format_time(item.modified)));
        if item.is_link {
            facts.add("Nature", "Lien symbolique");
        }
        facts
    }

    /// Rangée de boutons : ouvrir, copier le chemin, renommer, supprimer.
    fn item_actions(self: &Rc<Self>, item: &Entry) -> gtk4::Box {
        let actions = hbox(8);
        let row = gtk4::FlowBox::new();
        row.set_selection_mode(gtk4::SelectionMode::None);
        row.set_column_spacing(8);
        row.set_row_spacing(8);
        let (open_label, open_icon) = if item.is_dir { ("Ouvrir le dossier", "open") } else { ("Ouvrir", "file") };
        let primary = text_button(open_label, Some(open_icon), &["go"]);
        let (target, path, rename, delete) = (item.clone(), item.clone(), item.clone(), item.clone());
        on_click(&primary, self, move |app| app.open_item(&target));
        let copy = text_button("Copier le chemin", Some("copy"), &[]);
        on_click(&copy, self, move |app| app.copy_path(&path));
        let pen = text_button("Renommer", Some("pen"), &[]);
        on_click(&pen, self, move |app| app.rename_items(vec![rename.clone()]));
        let trash = text_button(if in_trash(&item.path) { "Supprimer définitivement" } else { "Mettre à la corbeille" }, Some("trash2"), &[]);
        on_click(&trash, self, move |app| app.delete_items(vec![delete.clone()]));
        for button in [primary, copy, pen, trash] {
            row.append(&button);
        }
        actions.append(&row);
        actions
    }

    /// Panneau d'un seul élément, avec l'aperçu rempli en arrière-plan.
    pub fn item_panel(self: &Rc<Self>, item: &Entry) -> Vec<Widget> {
        let facts = self.item_facts(item);
        let title = label(&item.name.to_uppercase(), &["h2"]);
        title.set_wrap(true);
        title.set_max_width_chars(1);
        title.set_wrap_mode(gtk4::pango::WrapMode::WordChar);
        title.set_hexpand(true);
        title.set_halign(gtk4::Align::Fill);
        let preview = vbox(8);
        preview.add_css_class("preview");
        preview.add_css_class("none");
        preview.set_halign(gtk4::Align::Fill);
        preview.set_valign(gtk4::Align::Center);
        preview.set_size_request(-1, 140);
        let click = gtk4::GestureClick::new();
        let weak = Rc::downgrade(self);
        click.connect_pressed(move |_, _, _, _| {
            if let Some(app) = weak.upgrade() {
                app.quick_look(None);
            }
        });
        preview.add_controller(click);
        self.fill_preview(&preview, item, kind_of(item), Some(facts.clone()), 300);
        let _ = ModifierType::empty();
        vec![title.upcast(), preview.upcast(), facts.grid.clone().upcast(), self.item_actions(item).upcast()]
    }
}
