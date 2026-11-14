//! Doublons d'un fichier par portée : même taille, puis même SHA-256 ; rang par date, nom le plus court, homonymes.

use std::collections::HashMap;
use std::path::Path;
use std::time::Instant;

use super::context::{DuplicateGroup, Duplicates};
use super::info_files::hash_of;
use super::scan::{Seen, scan_watch};
use super::types_rule::Watch;
use crate::root::Root;

/// Fichier de référence : chemin relatif, taille, date.
pub struct Target<'a> {
    pub path: &'a str,
    pub size: u64,
    pub mtime: f64,
}

/// Compare le fichier à ceux d'une portée : doublons exacts (hachage mis en cache), rang par date, nom le plus court, homonymes.
fn group_for(root: &Root, target: &Target, seen: &[Seen], cache: &mut HashMap<String, Option<String>>) -> DuplicateGroup {
    let name = target.path.rsplit('/').next().unwrap_or("");
    let others: Vec<&Seen> = seen.iter().filter(|s| !s.is_dir && s.path != target.path).collect();
    let mut hash_of_path = |path: &str| cache.entry(path.to_string()).or_insert_with(|| root.resolve_entry(path).ok().and_then(|full| hash_of(&full))).clone();
    let mine = hash_of_path(target.path);
    let same: Vec<&Seen> = others.iter().copied().filter(|s| s.size == target.size && mine.is_some() && hash_of_path(&s.path) == mine).collect();
    let key = |mtime: f64, path: &str| (mtime as i64, path.to_string());
    let (newest, oldest) = (
        same.iter().all(|s| key(s.mtime, &s.path) <= key(target.mtime, target.path)),
        same.iter().all(|s| key(s.mtime, &s.path) >= key(target.mtime, target.path)),
    );
    let shortest = same.iter().all(|s| (s.path.rsplit('/').next().unwrap_or("").len(), s.path.as_str()) >= (name.len(), target.path));
    let same_name = others.iter().filter(|s| s.path.rsplit('/').next() == Some(name)).count() as u32;
    DuplicateGroup { others: same.len() as u32, newest, oldest, shortest, same_name }
}

/// Fichiers d'une portée : un dossier, les dossiers surveillés, ou toute la racine ; `None` si le temps est écoulé.
fn gather(root: &Root, watches: &[Watch], deadline: Instant) -> Option<Vec<Seen>> {
    let mut out = Vec::new();
    watches.iter().all(|w| scan_watch(root, w, deadline, &mut out)).then_some(out)
}

/// Calcule les doublons du fichier dans les trois portées ; `None` si le budget de temps est dépassé.
pub fn duplicate_info(root: &Root, full: &Path, target: &Target, watches: &[Watch], budget: std::time::Duration) -> Option<Duplicates> {
    let deadline = Instant::now() + budget;
    let parent = root.relative(full.parent()?);
    let folder = [Watch { path: parent, recursive: false, max_depth: None, ignore_hidden: false }];
    let all = [Watch { path: String::new(), recursive: true, max_depth: None, ignore_hidden: false }];
    let mut cache = HashMap::new();
    let mut group = |scope: &[Watch]| gather(root, scope, deadline).map(|seen| group_for(root, target, &seen, &mut cache));
    Some(Duplicates { folder: group(&folder)?, watched: group(watches)?, root: group(&all)? })
}
