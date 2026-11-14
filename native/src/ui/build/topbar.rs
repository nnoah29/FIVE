//! Barre du haut : recherche, fichiers cachés, thème, paramètres, utilisateur.

use gtk4::prelude::*;
use gtk4::{Align, Button, Entry, WindowControls};

use crate::ui::buttons::icon_button;
use crate::ui::handles::TopUi;
use crate::ui::icon::icon;
use crate::ui::widgets::{hbox, label, text_button, vbox};

/// Zone de recherche : icône et champ de saisie.
fn search_field() -> Entry {
    let entry = Entry::new();
    entry.set_placeholder_text(Some("Chercher un dossier ou un fichier"));
    entry.set_hexpand(true);
    entry.set_width_chars(1);
    entry.set_valign(Align::Center);
    entry.add_css_class("search");
    entry
}

/// Pastille de l'utilisateur : initiale, nom et racine.
fn user_chip() -> (gtk4::Box, gtk4::Label, gtk4::Label, gtk4::Label, gtk4::Box) {
    let chip = hbox(12);
    chip.add_css_class("user");
    let avatar = label("?", &["avatar"]);
    avatar.set_xalign(0.5);
    avatar.set_halign(Align::Center);
    let (name, root) = (label("", &["who-name"]), label("", &["who-root"]));
    let who = vbox(0);
    who.set_valign(Align::Center);
    who.append(&name);
    who.append(&root);
    chip.append(&avatar);
    chip.append(&who);
    (chip, avatar, name, root, who)
}

/// Boutons de la barre : menu, fichiers cachés, thème, paramètres (avec info-bulle).
fn top_buttons() -> (Button, Button, Button, Button) {
    let menu = icon_button("menu", &[]);
    menu.set_visible(false);
    let (hidden, theme) = (icon_button("eye", &[]), icon_button("sun", &[]));
    let gear = text_button("Paramètres", Some("gear"), &[]);
    for (button, tip) in [(&menu, "Menu"), (&hidden, "Fichiers cachés ( . )"), (&theme, "Thème clair / sombre"), (&gear, "Paramètres ( , )")] {
        button.set_tooltip_text(Some(tip));
        button.set_valign(Align::Center);
    }
    (menu, hidden, theme, gear)
}

/// Barre du haut complète, sans comportement.
pub fn build_top() -> TopUi {
    let root = hbox(10);
    root.add_css_class("topbar");
    let (menu, hidden, theme, gear) = top_buttons();
    let search = search_field();
    let search_icon = icon("search");
    search_icon.add_css_class("search-ic");
    let (user, avatar, user_name, user_root, who) = user_chip();
    user.set_valign(Align::Center);
    let parts = [
        menu.upcast_ref::<gtk4::Widget>(),
        search_icon.upcast_ref(),
        search.upcast_ref(),
        hidden.upcast_ref(),
        theme.upcast_ref(),
        gear.upcast_ref(),
        user.upcast_ref(),
    ];
    parts.iter().for_each(|widget| root.append(*widget));
    let controls = WindowControls::new(gtk4::PackType::End);
    controls.set_valign(Align::Center);
    root.append(&controls);
    TopUi { root, menu, search, hidden, theme, gear, avatar, user_name, user_root, user, who }
}
