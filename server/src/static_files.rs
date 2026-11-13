//! Interface web : version embarquée dans le binaire, ou dossier choisi avec `--web`.

use std::fs;
use std::path::{Component, Path};

use five_engine::refus::{Refus, Resultat};

use crate::reply::{Reply, bytes};
use crate::state::State;

const EMBEDDED: &[(&str, &str, &[u8])] = &[
    ("index.html", "text/html; charset=utf-8", include_bytes!("../../web/index.html")),
    ("style.css", "text/css; charset=utf-8", include_bytes!("../../web/style.css")),
    ("app.js", "text/javascript; charset=utf-8", include_bytes!("../../web/app.js")),
    ("plus.js", "text/javascript; charset=utf-8", include_bytes!("../../web/plus.js")),
    ("boot.js", "text/javascript; charset=utf-8", include_bytes!("../../web/boot.js")),
    ("fonts/Anton-Regular.ttf", "font/ttf", include_bytes!("../../web/fonts/Anton-Regular.ttf")),
    ("fonts/HankenGrotesk-Regular.ttf", "font/ttf", include_bytes!("../../web/fonts/HankenGrotesk-Regular.ttf")),
    ("fonts/HankenGrotesk-Medium.ttf", "font/ttf", include_bytes!("../../web/fonts/HankenGrotesk-Medium.ttf")),
    ("fonts/HankenGrotesk-SemiBold.ttf", "font/ttf", include_bytes!("../../web/fonts/HankenGrotesk-SemiBold.ttf")),
    ("fonts/HankenGrotesk-Bold.ttf", "font/ttf", include_bytes!("../../web/fonts/HankenGrotesk-Bold.ttf")),
    ("fonts/JetBrainsMono-Regular.ttf", "font/ttf", include_bytes!("../../web/fonts/JetBrainsMono-Regular.ttf")),
    ("fonts/JetBrainsMono-Medium.ttf", "font/ttf", include_bytes!("../../web/fonts/JetBrainsMono-Medium.ttf")),
    ("fonts/JetBrainsMono-Bold.ttf", "font/ttf", include_bytes!("../../web/fonts/JetBrainsMono-Bold.ttf")),
];

/// Type de contenu d'un fichier de l'interface d'après son extension.
fn kind_of(name: &str) -> &'static str {
    match name.rsplit_once('.').map(|(_, ext)| ext) {
        Some("html") => "text/html; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("js") => "text/javascript; charset=utf-8",
        Some("ttf") => "font/ttf",
        Some("svg") => "image/svg+xml",
        _ => "application/octet-stream",
    }
}

/// Lit un fichier du dossier `--web` ; refuse tout chemin qui en sort.
fn from_disk(folder: &Path, name: &str) -> Option<Vec<u8>> {
    let safe = Path::new(name).components().all(|part| matches!(part, Component::Normal(_)));
    if safe { fs::read(folder.join(name)).ok() } else { None }
}

/// Sert un fichier de l'interface, sans mise en cache.
pub fn static_file(state: &State, path: &str) -> Resultat<Reply> {
    let name = if path == "/" { "index.html" } else { path.trim_start_matches('/') };
    let body = match &state.args.web {
        Some(folder) => from_disk(folder, name),
        None => EMBEDDED.iter().find(|(known, _, _)| *known == name).map(|(_, _, data)| data.to_vec()),
    };
    let body = body.ok_or_else(|| Refus::new(404, "Introuvable"))?;
    Ok(bytes(200, kind_of(name), body, &[("Cache-Control", "no-cache")]))
}
