//! Recherche de doublons : même taille, puis même empreinte SHA-256, avec budget de temps.

use std::collections::{BTreeMap, HashMap};
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::root::Root;

const MAX_GROUPS: usize = 150;

/// Regroupe les fichiers réguliers d'au moins `minimum` octets par taille, sans entrer dans les dossiers cachés.
fn collect_by_size(full: &Path, minimum: u64, deadline: Instant) -> (HashMap<u64, Vec<PathBuf>>, bool) {
    let (mut sizes, mut pending): (HashMap<u64, Vec<PathBuf>>, _) = (HashMap::new(), vec![full.to_path_buf()]);
    while let Some(folder) = pending.pop() {
        for entry in fs::read_dir(&folder).into_iter().flatten().flatten() {
            let Ok(kind) = entry.file_type() else { continue };
            let hidden = entry.file_name().to_string_lossy().starts_with('.');
            if kind.is_dir() && !hidden {
                pending.push(entry.path());
            } else if kind.is_file() && entry.metadata().is_ok_and(|m| m.len() >= minimum) {
                sizes.entry(entry.metadata().map_or(0, |m| m.len())).or_default().push(entry.path());
            }
        }
        if Instant::now() > deadline {
            return (sizes, true);
        }
    }
    (sizes, false)
}

/// Empreinte SHA-256 d'un fichier ; `None` si le temps est écoulé.
fn file_hash(path: &Path, deadline: Instant) -> std::io::Result<Option<String>> {
    let (mut file, mut digest, mut block) = (File::open(path)?, Sha256::new(), vec![0u8; 1 << 20]);
    loop {
        if Instant::now() > deadline {
            return Ok(None);
        }
        let read = file.read(&mut block)?;
        if read == 0 {
            return Ok(Some(format!("{:x}", digest.finalize())));
        }
        digest.update(&block[..read]);
    }
}

/// Sépare des fichiers de même taille en groupes d'empreinte identique ; le booléen signale un temps épuisé.
fn identical_groups(paths: &[PathBuf], deadline: Instant) -> (Vec<Vec<PathBuf>>, bool) {
    let mut by_hash: BTreeMap<String, Vec<PathBuf>> = BTreeMap::new();
    for path in paths {
        match file_hash(path, deadline) {
            Ok(Some(hash)) => by_hash.entry(hash).or_default().push(path.clone()),
            Ok(None) => return (Vec::new(), true),
            Err(_) => continue,
        }
    }
    (by_hash.into_values().filter(|same| same.len() > 1).collect(), false)
}

/// Groupes de fichiers identiques sous `full`, du plus gaspilleur au moins gaspilleur.
pub fn find_duplicates(root: &Root, full: &Path, minimum: u64, budget: Duration) -> Value {
    let deadline = Instant::now() + budget;
    let (sizes, mut partial) = collect_by_size(full, minimum, deadline);
    let mut groups: Vec<(u64, Vec<String>)> = Vec::new();
    for (size, paths) in sizes.iter().filter(|(_, paths)| paths.len() > 1) {
        let (found, late) = identical_groups(paths, deadline);
        partial |= late;
        for same in found {
            let mut names: Vec<String> = same.iter().map(|p| root.relative(p)).collect();
            names.sort();
            groups.push((*size, names));
        }
    }
    let waste = |g: &(u64, Vec<String>)| g.0 * (g.1.len() as u64 - 1);
    groups.sort_by_key(|g| std::cmp::Reverse(waste(g)));
    let wasted: u64 = groups.iter().map(waste).sum();
    let shown: Vec<Value> = groups.iter().take(MAX_GROUPS).map(|(size, files)| json!({"size": size, "files": files})).collect();
    json!({"groups": shown, "partial": partial, "wasted": wasted})
}
