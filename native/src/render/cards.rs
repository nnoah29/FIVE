//! Rendu des cartes de dossiers.

use std::rc::Rc;

use five_engine::entry::Entry;
use gtk4::prelude::*;
use gtk4::{Align, Button, Overlay};

use crate::app::menu_place::MenuPlace;
use crate::app::shared::{App, on_click};
use crate::model::format::{format_date, group_digits};
use crate::model::sorting::sort_items;
use crate::ui::buttons::dots_button;
use crate::ui::classes::clear;
use crate::ui::icon::icon_with_stroke;
use crate::ui::widgets::{hbox, label, vbox};

/// Texte du nombre d'éléments d'un dossier : `12 éléments`, `1 élément` ou `—`.
fn count_text(entry: &Entry) -> String {
    match entry.children {
        Some(Some(n)) => format!("{} {}", group_digits(n), if n > 1 { "éléments" } else { "élément" }),
        _ => "—".to_string(),
    }
}

/// Nom d'un dossier : une seule ligne, coupée par « … » plutôt que d'élargir la carte.
fn card_title(name: &str) -> gtk4::Label {
    let title = label(name, &["card-name"]);
    title.set_ellipsize(gtk4::pango::EllipsizeMode::End);
    title.set_max_width_chars(1);
    title.set_hexpand(true);
    title.set_halign(Align::Fill);
    title
}

impl App {
    /// Zone cliquable de la carte : icône, nom, badge Git, effectif et date.
    fn card_main(self: &Rc<Self>, dir: &Entry) -> Button {
        let button = Button::new();
        button.add_css_class("card-main");
        button.set_tooltip_text(Some(&dir.name));
        let column = vbox(22);
        let glyph = icon_with_stroke(if dir.is_link { "link" } else { "folder" }, 26, 1.5);
        glyph.set_halign(Align::Start);
        column.append(&glyph);
        let name = hbox(0);
        name.set_hexpand(true);
        name.append(&card_title(&dir.name));
        name.append(&self.git_badge(dir));
        let meta = hbox(8);
        let (count, date) = (label(&count_text(dir), &["card-meta"]), label(&format_date(dir.modified, self.set.borrow().full_dates), &["card-pill"]));
        date.set_hexpand(true);
        date.set_halign(Align::End);
        meta.append(&count);
        meta.append(&date);
        let text = vbox(6);
        text.append(&name);
        text.append(&meta);
        column.append(&text);
        button.set_child(Some(&column));
        let target = dir.path.clone();
        on_click(&button, self, move |app| app.go(&target));
        button
    }

    /// Carte d'un dossier : zone principale et bouton d'actions en haut à droite.
    fn card_for(self: &Rc<Self>, dir: &Entry) -> Overlay {
        let dots = dots_button();
        dots.set_halign(Align::End);
        dots.set_valign(Align::Start);
        dots.set_margin_top(6);
        dots.set_margin_end(6);
        let (entry, weak) = (dir.clone(), Rc::downgrade(self));
        dots.connect_clicked(move |button| {
            if let Some(app) = weak.upgrade() {
                app.open_menu_at(button, MenuPlace::BelowEnd, &entry, None);
            }
        });
        let card = Overlay::new();
        card.add_css_class("card");
        card.set_size_request(self.card_width(), -1);
        card.set_hexpand(false);
        card.set_halign(Align::Start);
        card.set_child(Some(&self.card_main(dir)));
        card.add_overlay(&dots);
        card
    }

    /// Largeur d'une carte : 19 % de la zone (entre 168 et 240 px), ou 70 % sur fenêtre étroite.
    pub(super) fn card_width(&self) -> i32 {
        let available = (self.ui.window.width() - if self.narrow_side() { 32 } else { 248 + 64 }).max(300) as f64;
        if self.narrow_side() { (available * 0.7) as i32 } else { (available * 0.19).clamp(168.0, 240.0) as i32 }
    }

    /// Reconstruit les cartes (aucune en recherche ni en vue virtuelle).
    pub fn render_cards(self: &Rc<Self>) {
        clear(&self.ui.folders.cards);
        let dirs: Vec<Entry> = {
            let st = self.st.borrow();
            if st.list_all() { Vec::new() } else { sort_items(&st.items.iter().filter(|i| i.is_dir).cloned().collect::<Vec<_>>(), st.sort, st.ascending) }
        };
        dirs.iter().for_each(|dir| self.ui.folders.cards.append(&self.card_for(dir)));
        let all = self.st.borrow().list_all();
        if dirs.is_empty() && !all {
            self.ui.folders.cards.append(&self.empty_card());
        }
        if let Some(scroll) = self.ui.folders.cards.parent().and_then(|p| p.parent()) {
            scroll.set_visible(!all);
        }
    }
}
