//! Vues virtuelles : documents récents, favoris GTK, montages réseau.

use std::fs;
use std::path::{Path, PathBuf};

use crate::entry::Entry;
use crate::mounts::read_mounts;
use crate::realpath::real_path;
use crate::root::Root;
use crate::urlcodec::decode;
use crate::xbel::bookmarks;

const NETWORK_TYPES: [&str; 8] = ["nfs", "nfs4", "cifs", "smb3", "fuse.sshfs", "fuse.rclone", "davfs", "fuse.davfs"];
const MAX_RECENT: usize = 200;

/// Chemin local d'une adresse `file://`, `None` pour tout autre schéma.
pub fn local_path(uri: &str) -> Option<PathBuf> {
    let rest = uri.strip_prefix("file://")?;
    let path = rest.find('/').map_or("", |at| &rest[at..]);
    Some(real_path(&PathBuf::from(decode(path))))
}

/// Décrit l'élément situé à `full`, avec son chemin relatif.
fn item_at(root: &Root, full: &Path) -> Option<Entry> {
    let name = full.file_name().map_or_else(|| full.to_string_lossy().into_owned(), |n| n.to_string_lossy().into_owned());
    Entry::describe(root, full, &name)
}

/// Documents récemment utilisés, sous la racine, du plus récent au plus ancien.
pub fn recent_items(root: &Root) -> Vec<Entry> {
    let xml = fs::read_to_string(root.home().join(".local/share/recently-used.xbel")).unwrap_or_default();
    let mut seen: Vec<PathBuf> = Vec::new();
    for (href, _) in bookmarks(&xml) {
        let Some(full) = local_path(&href) else { continue };
        if !seen.contains(&full) && root.inside(&full) && full.exists() {
            seen.push(full);
        }
        if seen.len() >= MAX_RECENT {
            break;
        }
    }
    seen.iter().filter_map(|full| item_at(root, full)).collect()
}

/// Favoris GTK existants sous la racine, sans doublon.
pub fn favorite_items(root: &Root) -> Vec<Entry> {
    let mut seen: Vec<PathBuf> = Vec::new();
    for name in [".config/gtk-3.0/bookmarks", ".gtk-bookmarks"] {
        let text = fs::read_to_string(root.home().join(name)).unwrap_or_default();
        for line in text.lines().filter(|l| !l.trim().is_empty()) {
            let uri = line.split(' ').next().unwrap_or("");
            let Some(full) = local_path(uri) else { continue };
            if root.inside(&full) && full.exists() && !seen.contains(&full) {
                seen.push(full);
            }
        }
    }
    seen.iter().filter_map(|full| item_at(root, full)).collect()
}

/// Dossiers montés par le réseau (NFS, SMB, SSHFS…) situés sous la racine.
pub fn network_items(root: &Root) -> Vec<Entry> {
    read_mounts()
        .into_iter()
        .filter(|m| NETWORK_TYPES.contains(&m.fstype.as_str()))
        .map(|m| PathBuf::from(m.point))
        .filter(|full| root.inside(full) && full.is_dir())
        .filter_map(|full| item_at(root, &full))
        .collect()
}
