//! Lectures du moteur pour l'interface : emplacement courant, emplacements, disques.

use five_engine::disks::{disk_space, mounted_disks};
use five_engine::entry::Entry;
use five_engine::listing::list_dir;
use five_engine::places::existing_places;
use five_engine::refus::Refus;
use five_engine::root::Root;
use five_engine::search::search_dir;
use five_engine::views::{favorite_items, network_items, recent_items};

use crate::state::{Disk, Place};

/// Contenu d'un emplacement avec l'espace disque disponible.
pub struct Listing {
    pub path: String,
    pub items: Vec<Entry>,
    pub free: u64,
    pub total: u64,
}

/// Lit un dossier, une recherche ou une vue virtuelle ; `Err` porte un message en français.
pub fn read_location(root: &Root, path: &str, view: Option<&str>, query: &str, hidden: bool) -> Result<Listing, String> {
    if let Some(name) = view {
        let items = match name {
            "recent" => recent_items(root),
            "favorites" => favorite_items(root),
            "network" => network_items(root),
            _ => return Err("Vue inconnue".to_string()),
        };
        let (free, total) = disk_space(root.path());
        let items = items.into_iter().filter(|i| hidden || !i.name.starts_with('.')).collect();
        return Ok(Listing { path: String::new(), items, free, total });
    }
    let full = root.resolve(path).map_err(|r| r.message)?;
    let needle = query.trim();
    let items = if needle.is_empty() { list_dir(root, &full, hidden).map_err(|e| Refus::from(e).message)? } else { search_dir(root, &full, needle, hidden, false) };
    let (free, total) = disk_space(&full);
    Ok(Listing { path: root.relative(&full), items, free, total })
}

/// Emplacements de la barre latérale.
pub fn read_places(root: &Root) -> Vec<Place> {
    let text = |value: &serde_json::Value, key: &str| value[key].as_str().unwrap_or("").to_string();
    existing_places(root).iter().map(|p| Place { id: text(p, "id"), label: text(p, "label"), path: text(p, "path") }).collect()
}

/// Disques montés.
pub fn read_disks(root: &Root) -> Vec<Disk> {
    let text = |value: &serde_json::Value, key: &str| value[key].as_str().unwrap_or("").to_string();
    mounted_disks(root)
        .iter()
        .map(|d| Disk { name: text(d, "n"), mount: text(d, "m"), fs: text(d, "fs"), total: d["t"].as_u64().unwrap_or(0), free: d["f"].as_u64().unwrap_or(0), path: d["p"].as_str().map(String::from) })
        .collect()
}
