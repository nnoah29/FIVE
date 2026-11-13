//! Décodage et affichage d'images pour les aperçus.

use std::path::PathBuf;

use gtk4::gdk::{MemoryFormat, MemoryTexture};
use gtk4::gdk_pixbuf::Pixbuf;
use gtk4::prelude::*;
use gtk4::{Box as GtkBox, ContentFit, Picture};

use crate::ui::classes::clear;

/// Image réduite en pixels bruts, avec ses dimensions d'origine.
pub type Decoded = Option<((i32, i32), (i32, i32, i32, bool, gtk4::glib::Bytes))>;

/// Lit l'image réduite à `limit` pixels de côté et ses dimensions d'origine.
pub fn decode_image(path: PathBuf, limit: i32) -> Decoded {
    let (_, width, height) = Pixbuf::file_info(&path)?;
    let pixbuf = Pixbuf::from_file_at_scale(&path, limit, limit, true).ok()?;
    Some(((width, height), (pixbuf.width(), pixbuf.height(), pixbuf.rowstride(), pixbuf.has_alpha(), pixbuf.read_pixel_bytes())))
}

/// Montre une image décodée dans la zone.
pub fn show_picture(zone: &GtkBox, pixels: (i32, i32, i32, bool, gtk4::glib::Bytes), height: i32) {
    let (width, tall, stride, alpha, bytes) = pixels;
    clear(zone);
    zone.remove_css_class("none");
    let picture = Picture::new();
    picture.set_content_fit(ContentFit::Contain);
    picture.set_can_shrink(true);
    picture.set_size_request(-1, height.min(tall.max(120)));
    picture.set_hexpand(true);
    let format = if alpha { MemoryFormat::R8g8b8a8 } else { MemoryFormat::R8g8b8 };
    picture.set_paintable(Some(&MemoryTexture::new(width, tall, format, &bytes, stride as usize)));
    zone.append(&picture);
}
