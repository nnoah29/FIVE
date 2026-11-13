//! Routes d'information : emplacements, réglages, disques, transferts, vues virtuelles.

use std::collections::HashMap;

use five_engine::disks::{disk_space, mounted_disks};
use five_engine::places::{TRASH, existing_places};
use five_engine::refus::{Refus, Resultat};
use five_engine::views::{favorite_items, network_items, recent_items};
use serde_json::json;

use crate::reply::{Reply, json_reply};
use crate::state::State;

/// Réponse de `/api/places`.
pub fn places(state: &State) -> Reply {
    json_reply(200, &json!({"places": existing_places(&state.root), "root": state.root.path(), "user": state.user}))
}

/// Réponse de `/api/info`.
pub fn info(state: &State) -> Reply {
    let engine = format!("Rust · five {}", env!("CARGO_PKG_VERSION"));
    json_reply(200, &json!({"root": state.root.path(), "port": state.port, "trash": TRASH, "engine": engine, "user": state.user}))
}

/// Réponse de `/api/disks` et `/api/jobs`.
pub fn disks_or_jobs(state: &State, name: &str) -> Reply {
    if name == "disks" {
        return json_reply(200, &json!({"disks": mounted_disks(&state.root)}));
    }
    json_reply(200, &json!({"jobs": state.jobs.view()}))
}

/// Réponse de `/api/view` : récents, favoris ou réseau, filtrés selon les fichiers cachés.
pub fn view(state: &State, query: &HashMap<String, String>) -> Resultat<Reply> {
    let items = match query.get("v").map(String::as_str) {
        Some("recent") => recent_items(&state.root),
        Some("favorites") => favorite_items(&state.root),
        Some("network") => network_items(&state.root),
        _ => return Err(Refus::new(404, "Vue inconnue")),
    };
    let hidden = query.get("h").is_some_and(|h| h == "1");
    let shown: Vec<_> = items.iter().filter(|i| hidden || !i.name.starts_with('.')).map(|i| i.to_json()).collect();
    let (free, total) = disk_space(state.root.path());
    Ok(json_reply(200, &json!({"path": "", "items": shown, "free": free, "total": total})))
}
