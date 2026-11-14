//! Assemblage de la fenêtre : barre latérale, haut, contenu, paramètres, couches flottantes.

use gtk4::prelude::*;
use gtk4::{Align, ApplicationWindow, Overlay, PolicyType, ScrolledWindow, Stack, WindowHandle};

use super::{files::build_files, folders::build_folders, sidebar::build_side, topbar::build_top, work::build_work};
use crate::ui::handles::Ui;
use crate::ui::widgets::{hbox, label, vbox};

/// Zone des fichiers avec le panneau de droite posé par-dessus, à droite.
fn lower_area(files: &gtk4::Box, work: &Overlay) -> Overlay {
    let lower = Overlay::new();
    files.set_margin_end(0);
    let scroller = ScrolledWindow::builder().hscrollbar_policy(PolicyType::External).vscrollbar_policy(PolicyType::Never).child(files).build();
    lower.set_child(Some(&scroller));
    work.set_halign(Align::End);
    lower.add_overlay(work);
    lower.set_vexpand(true);
    lower
}

/// Pile à deux pages : le contenu et la page Paramètres (défilante).
fn settings_stack(content: &gtk4::Box) -> (gtk4::Box, Stack) {
    let settings = vbox(0);
    settings.add_css_class("settings");
    let scroll = ScrolledWindow::builder().hscrollbar_policy(PolicyType::Never).child(&settings).build();
    let stack = Stack::new();
    stack.add_named(content, Some("content"));
    stack.add_named(&scroll, Some("settings"));
    stack.set_vexpand(true);
    (settings, stack)
}

/// Couches flottantes : message temporaire et file de transferts.
fn floating(overlay: &Overlay) -> (gtk4::Label, gtk4::Box) {
    let toast = label("", &["toast"]);
    toast.set_halign(Align::Center);
    toast.set_valign(Align::End);
    toast.set_margin_bottom(24);
    toast.set_can_target(false);
    let transfers = vbox(8);
    transfers.add_css_class("transfers");
    transfers.set_halign(Align::End);
    transfers.set_valign(Align::End);
    transfers.set_width_request(360);
    transfers.set_visible(false);
    overlay.add_overlay(&toast);
    overlay.add_overlay(&transfers);
    (toast, transfers)
}

/// Construit toute la fenêtre sans comportement : les boutons sont câblés ensuite.
pub fn build_ui(app: &gtk4::Application) -> Ui {
    let window = ApplicationWindow::builder().application(app).title("FIVE").default_width(1500).default_height(900).build();
    window.add_css_class("five");
    window.set_titlebar(Some(&gtk4::Box::new(gtk4::Orientation::Vertical, 0)));
    let (side, top, folders, files, work) = (build_side(), build_top(), build_folders(), build_files(), build_work());
    let lower = lower_area(&files.root, &work.root);
    let content = vbox(0);
    content.append(&folders.root);
    content.append(&lower);
    let (settings, stack) = settings_stack(&content);
    let main = vbox(0);
    main.set_hexpand(true);
    main.append(&WindowHandle::builder().child(&top.root).build());
    main.append(&stack);
    let side_scroll = ScrolledWindow::builder().hscrollbar_policy(PolicyType::Never).child(&side.root).build();
    side_scroll.set_size_request(248, -1);
    let page = hbox(0);
    page.append(&side_scroll);
    page.append(&main);
    let overlay = Overlay::new();
    overlay.set_child(Some(&page));
    let (toast, transfers) = floating(&overlay);
    let scrim = vbox(0);
    scrim.add_css_class("scrim");
    scrim.set_visible(false);
    overlay.add_overlay(&scrim);
    window.set_child(Some(&overlay));
    Ui { window, overlay, side, side_scroll, page, scrim, top, folders, files, work, stack, content, settings, toast, transfers, lower }
}
