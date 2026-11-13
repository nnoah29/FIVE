//! Panneau de droite : espace du dossier ou aperçu de la sélection.

use gtk4::prelude::*;
use gtk4::{Align, PolicyType, ScrolledWindow};

use crate::ui::buttons::icon_button;
use crate::ui::handles::WorkUi;
use crate::ui::widgets::vbox;

/// Panneau de droite vide, de 360 pixels de large.
pub fn build_work() -> WorkUi {
    let root = vbox(0);
    root.add_css_class("work");
    root.set_size_request(360, -1);
    let body = vbox(18);
    body.add_css_class("work-body");
    let scroll = ScrolledWindow::builder().hscrollbar_policy(PolicyType::Never).vexpand(true).child(&body).build();
    let close = icon_button("x", &[]);
    close.set_halign(Align::End);
    close.set_valign(Align::Start);
    close.set_margin_top(12);
    close.set_margin_end(12);
    close.set_visible(false);
    root.append(&scroll);
    WorkUi { root, body, close }
}
