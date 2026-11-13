//! Lignes du tableau (vue liste) : case, nom, type, taille, date, actions.

use std::rc::Rc;

use five_engine::entry::Entry;
use gtk4::gdk::ModifierType;
use gtk4::prelude::*;
use gtk4::{Box as GtkBox, GestureClick, Widget};

use super::table::column_widths;
use crate::app::shared::{App, on_click};
use crate::model::format::{format_date, format_size};
use crate::model::kinds::{kind_class, kind_label, kind_of};
use crate::model::paths::{name_parts, parent_of};
use crate::ui::buttons::dots_button;
use crate::ui::classes::fixed;
use crate::ui::icon::icon_with_stroke;
use crate::ui::widgets::{hbox, label, vbox};

/// Câble un clic sur une ligne ou une tuile : simple = sélection, double = ouverture, droit = menu.
pub fn attach_press(app: &Rc<App>, widget: &impl IsA<Widget>, item: &Entry, index: usize) {
    let click = GestureClick::new();
    click.set_button(0);
    let (weak, item, anchor) = (Rc::downgrade(app), item.clone(), widget.clone().upcast::<Widget>());
    click.connect_pressed(move |gesture, presses, _, _| {
        let Some(app) = weak.upgrade() else { return };
        let mods = gesture.current_event_state();
        match (gesture.current_button(), presses) {
            (3, _) => app.open_menu_at(&anchor, &item, Some(index)),
            (1, 2) => app.open_index(index),
            (1, _) => app.select(index as isize, mods),
            _ => {}
        }
    });
    widget.add_controller(click);
}

/// Cellule du type : pastille de couleur et libellé de la famille.
fn tag_cell(kind: &str) -> GtkBox {
    let tag = hbox(8);
    let dot = GtkBox::new(gtk4::Orientation::Horizontal, 0);
    dot.add_css_class("tag-dot");
    dot.add_css_class(kind_class(kind));
    dot.set_valign(gtk4::Align::Center);
    tag.append(&dot);
    tag.append(&label(kind_label(kind), &["tag"]));
    tag
}

impl App {
    /// Cellule du nom : radical, extension atténuée, badge Git, sous-titre en recherche.
    fn name_cell(self: &Rc<Self>, item: &Entry, sub: Option<String>) -> GtkBox {
        let (stem, tail) = name_parts(&item.name);
        let line = hbox(0);
        let title = label(&stem, &["name"]);
        title.set_ellipsize(gtk4::pango::EllipsizeMode::End);
        title.set_tooltip_text(Some(&item.name));
        line.append(&title);
        if !tail.is_empty() {
            line.append(&label(&tail, &["name-em"]));
        }
        line.append(&self.git_badge(item));
        let cell = vbox(0);
        cell.set_hexpand(true);
        cell.set_valign(gtk4::Align::Center);
        cell.append(&line);
        if let Some(text) = sub {
            cell.append(&label(&text, &["name-sub"]));
        }
        cell
    }

    /// Case de sélection : carré qui se coche.
    fn check_box(self: &Rc<Self>, item: &Entry, index: usize) -> GtkBox {
        let check = gtk4::Button::new();
        check.add_css_class("check");
        check.set_child(Some(&icon_with_stroke("check", 11, 2.4)));
        check.set_valign(gtk4::Align::Center);
        let _ = item;
        on_click(&check, self, move |app| app.select(index as isize, ModifierType::CONTROL_MASK));
        fixed(34, &check)
    }

    /// Ligne complète d'un élément pour la vue liste.
    pub fn row_for(self: &Rc<Self>, item: &Entry, index: usize) -> GtkBox {
        let kind = kind_of(item);
        let (full_dates, binary) = (self.set.borrow().full_dates, self.set.borrow().binary_units);
        let (tag_w, size_w, date_w) = column_widths(full_dates);
        let sub = self.st.borrow().list_all().then(|| format!("/{}", parent_of(&item.path)));
        let row = hbox(10);
        row.add_css_class("row");
        if self.st.borrow().picked.contains(&item.path) {
            row.add_css_class("selected");
        }
        row.append(&self.check_box(item, index));
        row.append(&self.name_cell(item, sub));
        row.append(&fixed(tag_w, &tag_cell(kind)));
        row.append(&fixed(size_w, &label(&if item.is_dir { "—".to_string() } else { format_size(item.size, binary) }, &["meta"])));
        row.append(&fixed(date_w, &label(&format_date(item.modified, full_dates), &["meta"])));
        let dots = dots_button();
        let (entry, weak) = (item.clone(), Rc::downgrade(self));
        dots.connect_clicked(move |button| {
            if let Some(app) = weak.upgrade() {
                app.open_menu_at(button, &entry, Some(index));
            }
        });
        row.append(&fixed(36, &dots));
        attach_press(self, &row, item, index);
        row
    }

    /// Toutes les lignes de la vue liste, dans l'ordre d'affichage.
    pub fn build_rows(self: &Rc<Self>) -> Vec<GtkBox> {
        let shown = self.st.borrow().shown.clone();
        shown.iter().enumerate().map(|(index, item)| self.row_for(item, index)).collect()
    }
}
