//! Choix de la couleur principale : pastilles et saisie hexadécimale.

use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{Button, Entry, FlowBox, SelectionMode};

use crate::app::shared::App;
use crate::model::accent::{SWATCHES, is_hex};
use crate::ui::classes::set_class;
use crate::ui::icon::icon_with_stroke;

/// Couleur actuellement choisie, en minuscules (l'orange d'origine si rien n'est choisi).
fn current(app: &App) -> String {
    let accent = app.set.borrow().accent.to_lowercase();
    if accent.is_empty() { SWATCHES[0].to_string() } else { accent }
}

impl App {
    /// Pastille de couleur : coche visible sur la couleur choisie.
    fn swatch(self: &Rc<Self>, at: usize, color: &str) -> Button {
        let button = Button::new();
        button.add_css_class("sw");
        button.add_css_class(&format!("sw-{at}"));
        button.set_child(Some(&icon_with_stroke("check", 16, 2.2)));
        set_class(&button, "on", current(self) == color);
        let (weak, color) = (Rc::downgrade(self), color.to_string());
        button.connect_clicked(move |_| {
            if let Some(app) = weak.upgrade() {
                let value = if color == SWATCHES[0] { String::new() } else { color.clone() };
                app.change_setting(move |s| s.accent = value);
            }
        });
        button
    }

    /// Saisie d'une couleur personnalisée : appliquée dès qu'elle est valide, sans redessiner la page.
    fn hex_entry(self: &Rc<Self>) -> Entry {
        let entry = Entry::new();
        entry.add_css_class("hex");
        entry.set_placeholder_text(Some("#personnalisée"));
        entry.set_max_length(7);
        let accent = current(self);
        entry.set_text(if SWATCHES.contains(&accent.as_str()) { "" } else { &accent });
        let weak = Rc::downgrade(self);
        entry.connect_changed(move |e| {
            let text = e.text().trim().to_string();
            set_class(e, "invalid", !text.is_empty() && !is_hex(&text));
            if let (true, Some(app)) = (is_hex(&text), weak.upgrade()) {
                app.change_setting_quiet(move |s| s.accent = text.to_lowercase());
            }
        });
        entry
    }

    /// Ensemble des pastilles et du champ hexadécimal.
    pub fn accent_control(self: &Rc<Self>) -> FlowBox {
        let flow = FlowBox::new();
        flow.set_selection_mode(SelectionMode::None);
        flow.set_column_spacing(12);
        flow.set_row_spacing(12);
        flow.set_max_children_per_line(20);
        for (at, color) in SWATCHES.iter().enumerate() {
            flow.append(&self.swatch(at, color));
        }
        flow.append(&self.hex_entry());
        flow
    }
}
