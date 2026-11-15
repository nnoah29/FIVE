//! Éléments de l'aperçu : image, début d'un texte, informations d'un fichier ou d'un dossier.

use std::fs;
use std::io::Read;
use std::path::Path;

use gtk4::prelude::*;
use gtk4::{ContentFit, Picture, PolicyType, ScrolledWindow, TextView, Widget, WrapMode};

use super::listing::Row;
use crate::model::format::{format_date, format_size};
use crate::ui::widgets::{label, vbox};

const IMAGE_LIMIT: u64 = 30 * 1024 * 1024;
const TEXT_BYTES: u64 = 4096;
const TEXT_LINES: usize = 40;

/// Début d'un fichier texte : `None` s'il contient autre chose que de l'UTF-8 lisible.
pub fn text_excerpt(path: &Path) -> Option<String> {
    let mut bytes = Vec::new();
    fs::File::open(path).ok()?.take(TEXT_BYTES).read_to_end(&mut bytes).ok()?;
    let text = match std::str::from_utf8(&bytes) {
        Ok(text) => text,
        Err(error) if error.valid_up_to() > 0 && bytes.len() as u64 == TEXT_BYTES => std::str::from_utf8(&bytes[..error.valid_up_to()]).ok()?,
        Err(_) => return None,
    };
    (!bytes.contains(&0) && !text.trim().is_empty()).then(|| text.lines().take(TEXT_LINES).collect::<Vec<_>>().join("\n"))
}

/// Image réduite pour tenir dans le panneau ; `None` si le fichier est trop gros ou illisible.
pub fn image_widget(row: &Row) -> Option<Widget> {
    let image = fs::metadata(&row.path).ok().filter(|m| m.len() <= IMAGE_LIMIT).is_some() && five_engine::mime::guess(&row.name).starts_with("image/");
    image.then(|| {
        let picture = Picture::for_filename(&row.path);
        picture.set_content_fit(ContentFit::ScaleDown);
        picture.set_size_request(-1, 200);
        picture.add_css_class("chooser-picture");
        picture.upcast()
    })
}

/// Zone de texte en lecture seule avec le début du fichier.
pub fn text_widget(excerpt: &str) -> Widget {
    let view = TextView::new();
    view.set_editable(false);
    view.set_cursor_visible(false);
    view.set_focusable(false);
    view.set_monospace(true);
    view.set_wrap_mode(WrapMode::WordChar);
    view.buffer().set_text(excerpt);
    view.add_css_class("chooser-excerpt");
    let scroll = ScrolledWindow::builder().hscrollbar_policy(PolicyType::Never).child(&view).build();
    scroll.set_size_request(-1, 220);
    scroll.upcast()
}

/// Informations affichées sous l'aperçu : type, taille ou contenu, date.
pub fn facts_box(row: &Row) -> gtk4::Box {
    let column = vbox(4);
    let name = label(&row.name, &["chooser-fact-name"]);
    name.set_ellipsize(gtk4::pango::EllipsizeMode::Middle);
    name.set_tooltip_text(Some(&row.name));
    let size = if row.is_dir { fs::read_dir(&row.path).map_or(String::new(), |d| format!("{} éléments", d.count())) } else { format_size(row.size, false) };
    let kind = if row.is_dir { "Dossier" } else { five_engine::mime::guess(&row.name) };
    column.append(&name);
    for text in [kind.to_string(), size, format_date(row.modified, true)] {
        column.append(&label(&text, &["dim", "chooser-fact"]));
    }
    column
}
