//! Fichier `portals.conf` de l'utilisateur : choisir FIVE comme sélecteur de fichiers, ou y renoncer.

/// Clé de l'interface FileChooser dans la section `[preferred]`.
pub const KEY: &str = "org.freedesktop.impl.portal.FileChooser";

/// Début et fin (exclue) des lignes de la section `[preferred]`, si elle existe.
fn preferred_bounds(lines: &[String]) -> Option<(usize, usize)> {
    let start = lines.iter().position(|l| l.trim() == "[preferred]")?;
    let end = lines.iter().skip(start + 1).position(|l| l.trim_start().starts_with('[')).map_or(lines.len(), |i| start + 1 + i);
    Some((start, end))
}

/// Position de la ligne `FileChooser=` dans la section `[preferred]`.
fn key_line(lines: &[String]) -> Option<usize> {
    let (start, end) = preferred_bounds(lines)?;
    (start + 1..end).find(|i| lines[*i].split_once('=').is_some_and(|(k, _)| k.trim() == KEY))
}

/// Vrai si le premier sélecteur choisi est FIVE.
pub fn is_enabled(text: &str) -> bool {
    let lines: Vec<String> = text.lines().map(String::from).collect();
    key_line(&lines).is_some_and(|i| lines[i].split_once('=').is_some_and(|(_, v)| v.trim().split(';').next() == Some("five")))
}

/// Retourne le texte avec FIVE choisi (`five;gtk`) ou retiré ; les autres lignes ne bougent pas.
pub fn set_chooser(text: &str, enabled: bool) -> String {
    let mut lines: Vec<String> = text.lines().map(String::from).collect();
    let wanted = format!("{KEY}=five;gtk");
    match (key_line(&lines), preferred_bounds(&lines), enabled) {
        (Some(i), _, true) => lines[i] = wanted,
        (None, Some((start, _)), true) => lines.insert(start + 1, wanted),
        (None, None, true) => lines.extend(["[preferred]".to_string(), wanted]),
        (Some(i), _, false) if is_enabled(text) => drop(lines.remove(i)),
        _ => {}
    }
    lines.join("\n") + "\n"
}
