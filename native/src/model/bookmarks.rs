//! Signets GTK (`~/.config/gtk-3.0/bookmarks`) : ajouter ou retirer un dossier dans le texte du fichier.

use five_engine::urlcodec::{decode, encode_path};

/// Chemin d'une ligne de signet `file:///chemin Nom`.
fn path_of(line: &str) -> Option<String> {
    let uri = line.split(' ').next()?;
    uri.strip_prefix("file://").map(decode)
}

/// Ajoute (`add`) ou retire un chemin absolu des signets ; sans doublon, les autres lignes sont conservées telles quelles.
pub fn toggle_bookmark(text: &str, path: &str, add: bool) -> String {
    let mut lines: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).collect();
    let present = lines.iter().any(|l| path_of(l).as_deref() == Some(path));
    let line = format!("file://{} {}", encode_path(path), path.rsplit('/').next().unwrap_or(""));
    if add && !present {
        lines.push(&line);
        return lines.join("\n") + "\n";
    }
    if !add {
        lines.retain(|l| path_of(l).as_deref() != Some(path));
    }
    if lines.is_empty() { String::new() } else { lines.join("\n") + "\n" }
}
