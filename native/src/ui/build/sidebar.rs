//! Barre latérale : marque, compteur, actions principales, emplacements, disques.

use gtk4::Align;
use gtk4::prelude::*;

use super::logo::logo;
use crate::ui::handles::SideUi;
use crate::ui::widgets::{hbox, label, mono, text_button, vbox};

/// Marque : logo et nom « FIVE » dont la fin est à la couleur principale.
fn brand() -> gtk4::Box {
    let row = hbox(12);
    row.add_css_class("side-pad");
    row.append(&logo());
    let name = hbox(0);
    name.set_valign(Align::Center);
    name.append(&label("FI", &["wordmark"]));
    name.append(&label("VE", &["wordmark", "wordmark-b"]));
    row.append(&name);
    row
}

/// Compteur d'éléments : grand nombre et libellé.
fn counter() -> (gtk4::Box, gtk4::Label, gtk4::Label) {
    let column = vbox(6);
    column.add_css_class("counter");
    column.add_css_class("side-pad");
    let (count, caption) = (label("0", &["big"]), mono("éléments", &["dim"]));
    column.append(&count);
    column.append(&caption);
    (column, count, caption)
}

/// Barre latérale complète, sans comportement (les boutons sont câblés plus tard).
pub fn build_side() -> SideUi {
    let root = vbox(0);
    root.add_css_class("side");
    root.append(&brand());
    let (counter, count, count_label) = counter();
    root.append(&counter);
    let new_folder = text_button("Nouveau dossier", Some("plus"), &["go", "block"]);
    let up = text_button("Dossier parent", Some("up"), &["block"]);
    up.set_margin_top(8);
    new_folder.add_css_class("side-pad");
    up.add_css_class("side-pad");
    let (places, disks) = (vbox(0), vbox(0));
    places.add_css_class("nav");
    disks.add_css_class("nav");
    [&new_folder, &up].iter().for_each(|b| root.append(*b));
    root.append(&places);
    root.append(&disks);
    SideUi { root, count, count_label, new_folder, up, places, disks }
}
