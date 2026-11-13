//! Routes qui travaillent sur un fichier : texte, contenu brut, archive.

use std::collections::HashMap;
use std::path::Path;

use five_engine::archive::list_archive;
use five_engine::mime;
use five_engine::refus::{Refus, Resultat};
use five_engine::text::read_text;
use five_engine::urlcodec::encode_path;

use crate::file_send::{Options, send_file};
use crate::reply::{Reply, json_reply};

/// Exige que le chemin soit un fichier ordinaire.
pub fn regular(full: &Path) -> Resultat<()> {
    if full.is_file() { Ok(()) } else { Err(Refus::new(400, "Ce n'est pas un fichier")) }
}

/// Réponse de `/api/text` et `/api/archive`.
pub fn inspect(name: &str, full: &Path) -> Resultat<Reply> {
    regular(full)?;
    let payload = if name == "text" { read_text(full)? } else { list_archive(full)? };
    Ok(json_reply(200, &payload))
}

/// Réponse de `/api/raw` : contenu brut, avec `dl=1` pour forcer le téléchargement.
pub fn raw(full: &Path, query: &HashMap<String, String>, range: Option<&str>) -> Resultat<Reply> {
    regular(full)?;
    let name = full.file_name().map_or_else(String::new, |n| n.to_string_lossy().into_owned());
    let kind = match mime::guess(&name) {
        "" => "application/octet-stream",
        known => known,
    };
    let download = query.get("dl").is_some_and(|d| d == "1");
    let disposition = download.then(|| format!("attachment; filename*=UTF-8''{}", encode_path(&name)));
    send_file(full, range, &Options { kind, disposition, sandbox: kind != "application/pdf", extra: &[] })
}
