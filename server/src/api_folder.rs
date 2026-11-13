//! Routes qui travaillent sur un dossier : liste, recherche, occupation, doublons, Git.

use std::collections::HashMap;
use std::path::Path;
use std::time::Duration;

use five_engine::disks::disk_space;
use five_engine::duplicates::find_duplicates;
use five_engine::git_marks::git_marks;
use five_engine::listing::list_dir;
use five_engine::refus::{Refus, Resultat};
use five_engine::search::search_dir;
use five_engine::usage::usage_of;
use serde_json::json;

use crate::reply::{Reply, json_reply};
use crate::state::State;

/// Exige que le chemin soit un dossier.
pub fn folder(full: &Path) -> Resultat<()> {
    if full.is_dir() { Ok(()) } else { Err(Refus::new(400, "Ce n'est pas un dossier")) }
}

/// Réponse de `/api/list` : contenu d'un dossier, ou résultats de recherche si `q` est renseigné.
pub fn list(state: &State, full: &Path, query: &HashMap<String, String>) -> Resultat<Reply> {
    let hidden = query.get("h").is_some_and(|h| h == "1");
    let needle = query.get("q").map_or("", |q| q.trim());
    let entries = if needle.is_empty() {
        list_dir(&state.root, full, hidden)?
    } else {
        search_dir(&state.root, full, needle, hidden, query.get("f").is_some_and(|f| f == "1"))
    };
    let items: Vec<_> = entries.iter().map(|e| e.to_json()).collect();
    let (free, total) = disk_space(full);
    Ok(json_reply(200, &json!({"path": state.root.relative(full), "items": items, "free": free, "total": total})))
}

/// Réponse de `/api/usage`, `/api/duplicates` et `/api/git`.
pub fn analysis(state: &State, name: &str, full: &Path, query: &HashMap<String, String>) -> Resultat<Reply> {
    folder(full)?;
    match name {
        "git" => Ok(json_reply(200, &json!({"marks": git_marks(full)}))),
        "usage" => Ok(json_reply(200, &usage_of(&state.root, full, Duration::from_secs(10))?)),
        _ => {
            let minimum = query.get("min").filter(|m| !m.is_empty()).map_or(Ok(1024), |m| m.parse::<u64>());
            let minimum = minimum.map_err(|_| Refus::new(400, "Paramètre min invalide"))?.max(1);
            Ok(json_reply(200, &find_duplicates(&state.root, full, minimum, Duration::from_secs(20))))
        }
    }
}
