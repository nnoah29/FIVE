//! Chargement des vignettes d'images en arrière-plan.

use std::rc::Rc;

use gtk4::Picture;
use gtk4::gdk::{MemoryFormat, MemoryTexture};
use gtk4::gdk_pixbuf::Pixbuf;
use gtk4::glib::Bytes;

use crate::app::shared::App;
use crate::app::work::spawn_work;

/// Pixels bruts d'une image réduite : largeur, hauteur, octets par ligne, alpha, données.
type Pixels = (i32, i32, i32, bool, Bytes);

/// Lit et réduit l'image à 276 × 184 au plus ; `None` si elle est illisible.
fn decode(path: std::path::PathBuf) -> Option<Pixels> {
    let pixbuf = Pixbuf::from_file_at_scale(path, 276, 184, true).ok()?;
    Some((pixbuf.width(), pixbuf.height(), pixbuf.rowstride(), pixbuf.has_alpha(), pixbuf.read_pixel_bytes()))
}

impl App {
    /// Charge l'image `rel` (chemin relatif à la racine) dans `picture`, sans bloquer l'interface.
    pub fn load_thumb(self: &Rc<Self>, rel: &str, picture: Picture) {
        let Ok(full) = self.root.resolve(rel) else { return };
        spawn_work(
            move || decode(full),
            move |pixels| {
                if let Some((width, height, stride, alpha, bytes)) = pixels {
                    let format = if alpha { MemoryFormat::R8g8b8a8 } else { MemoryFormat::R8g8b8 };
                    picture.set_paintable(Some(&MemoryTexture::new(width, height, format, &bytes, stride as usize)));
                }
            },
        );
    }
}
