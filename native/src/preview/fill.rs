//! Méthodes d'aperçu : image, média, archive, PDF, texte.

use std::path::PathBuf;
use std::process::Command;
use std::rc::Rc;

use five_engine::archive::list_archive;
use five_engine::text::read_text;
use gtk4::prelude::*;
use gtk4::{Box as GtkBox, MediaFile, Video};

use super::facts::Facts;
use super::images::{decode_image, show_picture};
use super::media::{none_state, show_text};
use crate::app::shared::App;
use crate::app::work::spawn_work;
use crate::model::format::{format_clock, format_size};
use crate::ui::classes::clear;

impl App {
    /// Aperçu d'une image : chargement hors du fil de l'interface, dimensions ajoutées à la fiche.
    pub fn fill_image(self: &Rc<Self>, zone: &GtkBox, full: PathBuf, facts: Option<Rc<Facts>>, height: i32) {
        let zone = zone.clone();
        spawn_work(
            move || decode_image(full, 1400),
            move |decoded| {
                if zone.root().is_none() {
                    return;
                }
                match decoded {
                    Some(((w, h), pixels)) => {
                        show_picture(&zone, pixels, height);
                        if let Some(facts) = facts {
                            facts.add("Dimensions", &format!("{w} × {h} px"));
                        }
                    }
                    None => none_state(&zone, "image", "Aperçu indisponible"),
                }
            },
        );
    }

    /// Aperçu vidéo ou audio : lecteur avec commandes ; la durée est ajoutée dès que le média est prêt.
    pub fn fill_media(self: &Rc<Self>, zone: &GtkBox, full: PathBuf, kind: &str, facts: Option<Rc<Facts>>, height: i32) {
        clear(zone);
        zone.remove_css_class("none");
        let file = MediaFile::for_filename(&full);
        let video = Video::new();
        video.set_media_stream(Some(&file));
        video.set_autoplay(false);
        video.set_size_request(-1, if kind == "audio" { 60 } else { height });
        video.set_hexpand(true);
        zone.append(&video);
        let (kind, facts) = (kind.to_string(), facts);
        file.connect_prepared_notify(move |stream| {
            let Some(facts) = &facts else { return };
            facts.add("Durée", &format_clock(stream.duration() as f64 / 1_000_000.0));
            if kind == "video" && stream.intrinsic_width() > 0 {
                facts.add("Dimensions", &format!("{} × {} px", stream.intrinsic_width(), stream.intrinsic_height()));
            }
        });
    }

    /// Aperçu d'une archive : liste des entrées avec leur taille.
    pub fn fill_archive(self: &Rc<Self>, zone: &GtkBox, full: PathBuf, height: i32) {
        let (zone, units) = (zone.clone(), self.set.borrow().binary_units);
        spawn_work(
            move || list_archive(&full).map_err(|r| r.message),
            move |result| {
                if zone.root().is_none() {
                    return;
                }
                match result {
                    Ok(data) => {
                        let rows = data["items"].as_array().cloned().unwrap_or_default();
                        let lines: Vec<String> = rows
                            .iter()
                            .map(|r| {
                                format!(
                                    "{:>9}  {}",
                                    if r["d"].as_bool().unwrap_or(false) { String::new() } else { format_size(r["s"].as_u64().unwrap_or(0), units) },
                                    r["n"].as_str().unwrap_or("")
                                )
                            })
                            .collect();
                        show_text(&zone, &(lines.join("\n") + if data["more"].as_bool().unwrap_or(false) { "\n…" } else { "" }), height);
                    }
                    Err(message) => none_state(&zone, "archive", &message),
                }
            },
        );
    }

    /// Aperçu d'un PDF : première page rendue par `pdftoppm` s'il est installé.
    pub fn fill_pdf(self: &Rc<Self>, zone: &GtkBox, full: PathBuf, height: i32) {
        let zone = zone.clone();
        spawn_work(
            move || {
                let out = std::env::temp_dir().join(format!("five-pdf-{}", std::process::id()));
                let ok = Command::new("pdftoppm")
                    .args(["-png", "-r", "70", "-f", "1", "-l", "1", "-singlefile"])
                    .arg(&full)
                    .arg(&out)
                    .status()
                    .is_ok_and(|s| s.success());
                let png = out.with_extension("png");
                let pixels = if ok { decode_image(png.clone(), 1000) } else { None };
                let _ = std::fs::remove_file(png);
                pixels
            },
            move |decoded| {
                if zone.root().is_none() {
                    return;
                }
                match decoded {
                    Some((_, pixels)) => show_picture(&zone, pixels, height),
                    None => none_state(&zone, "doc", "Aperçu PDF indisponible"),
                }
            },
        );
    }

    /// Aperçu texte : 200 Ko au plus, ou message si le fichier est binaire.
    pub fn fill_text(self: &Rc<Self>, zone: &GtkBox, full: PathBuf, kind: &str, height: i32) {
        let (zone, kind) = (zone.clone(), kind.to_string());
        spawn_work(
            move || read_text(&full).map_err(|e| five_engine::refus::Refus::from(e).message),
            move |result| {
                if zone.root().is_none() {
                    return;
                }
                match result {
                    Ok(data) if data["binary"].as_bool().unwrap_or(false) => none_state(&zone, &kind, "Aperçu indisponible : fichier binaire"),
                    Ok(data) => show_text(
                        &zone,
                        &(data["text"].as_str().unwrap_or("").to_string() + if data["truncated"].as_bool().unwrap_or(false) { "\n…" } else { "" }),
                        height,
                    ),
                    Err(message) => none_state(&zone, &kind, &message),
                }
            },
        );
    }
}
