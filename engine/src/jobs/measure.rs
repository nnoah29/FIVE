//! Mesure de ce qu'un transfert devra copier.

use std::fs;
use std::path::Path;

/// Taille cumulée et nombre de fichiers sous `path` (au moins 1), sans suivre les liens.
pub fn measure(path: &Path) -> (u64, u64) {
    let Ok(info) = fs::symlink_metadata(path) else { return (0, 1) };
    if !info.is_dir() {
        return (info.len(), 1);
    }
    let (mut size, mut count) = (0, 0);
    let mut pending = vec![path.to_path_buf()];
    while let Some(folder) = pending.pop() {
        for entry in fs::read_dir(&folder).into_iter().flatten().flatten() {
            match entry.file_type() {
                Ok(kind) if kind.is_dir() => pending.push(entry.path()),
                _ => {
                    size += entry.metadata().map_or(0, |m| m.len());
                    count += 1;
                }
            }
        }
    }
    (size, count.max(1))
}
