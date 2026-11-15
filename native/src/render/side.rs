//! Rendu de la barre latérale : compteur, emplacements, disques.

use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{Button, DrawingArea, Overlay};

use crate::app::shared::{App, on_click};
use crate::model::format::{format_size, group_digits};
use crate::state::Disk;
use crate::ui::classes::{clear, set_class};
use crate::ui::icon::icon;
use crate::ui::widgets::{hbox, label, vbox};

/// Barre d'occupation d'un disque, dessinée sur toute la largeur du bouton.
fn usage_bar(ratio: f64) -> DrawingArea {
    let bar = DrawingArea::new();
    bar.add_css_class("disk-bar-area");
    bar.set_content_height(2);
    bar.set_valign(gtk4::Align::End);
    bar.set_can_target(false);
    bar.set_draw_func(move |widget, cr, width, height| {
        let color = widget.color();
        cr.set_source_rgba(color.red() as f64, color.green() as f64, color.blue() as f64, 0.7);
        cr.rectangle(0.0, 0.0, width as f64 * ratio, height as f64);
        let _ = cr.fill();
    });
    bar
}

impl App {
    /// Bouton d'un disque : icône, nom, occupation en texte, barre en pied.
    fn disk_button(self: &Rc<Self>, disk: &Disk, current: bool) -> Overlay {
        let button = Button::new();
        button.add_css_class("navitem");
        button.add_css_class("disk");
        set_class(&button, "current", current);
        button.set_sensitive(disk.path.is_some());
        let units = self.set.borrow().binary_units;
        let text = format!("{} / {}", format_size(disk.total - disk.free, units), format_size(disk.total, units));
        let top = hbox(14);
        top.append(&icon("drive"));
        top.append(&label(&disk.name, &[]));
        let sub = label(&text, &["disk-sub"]);
        sub.set_margin_start(30);
        let column = vbox(2);
        column.append(&top);
        column.append(&sub);
        button.set_child(Some(&column));
        button.set_tooltip_text(Some(&format!("{} · {}", disk.mount, disk.fs)));
        let target = disk.path.clone().unwrap_or_default();
        on_click(&button, self, move |app| app.go(&target));
        let ratio = if disk.total > 0 { (disk.total - disk.free) as f64 / disk.total as f64 } else { 0.0 };
        let overlay = Overlay::new();
        overlay.set_child(Some(&button));
        overlay.add_overlay(&usage_bar(ratio));
        overlay
    }

    /// Réécrit compteur, état des boutons, emplacements et disques.
    pub fn render_side(self: &Rc<Self>) {
        let (count, searching, up, create) = {
            let st = self.st.borrow();
            (st.items.len(), st.searching(), !st.path.is_empty() && st.virtual_view.is_none(), st.can_create())
        };
        self.ui.side.count.set_text(&group_digits(count as u64));
        self.ui.side.count_label.set_text(if searching {
            "RÉSULTATS"
        } else if count > 1 {
            "ÉLÉMENTS"
        } else {
            "ÉLÉMENT"
        });
        self.ui.side.up.set_sensitive(up);
        self.ui.side.new_folder.set_sensitive(create);
        self.render_places();
        self.render_disks();
        self.render_dock();
    }

    /// Reconstruit la liste des disques montés.
    fn render_disks(self: &Rc<Self>) {
        clear(&self.ui.side.disks);
        let (disks, path, view) = {
            let st = self.st.borrow();
            (st.disks.clone(), st.path.clone(), st.virtual_view.clone())
        };
        self.ui.side.disks.append(&label("DISQUES", &["nav-title"]));
        for disk in &disks {
            let here = view.is_none() && disk.path.as_ref().is_some_and(|p| path == *p || path.starts_with(&format!("{p}/")));
            self.ui.side.disks.append(&self.disk_button(disk, here));
        }
    }
}
