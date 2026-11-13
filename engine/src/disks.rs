//! Disques montés et espace disponible.

use std::path::Path;

use rustix::fs::statvfs;
use serde_json::{Value, json};

use crate::mounts::read_mounts;
use crate::root::Root;

const IGNORED_POINTS: [&str; 3] = ["/boot", "/var/lib", "/snap"];

/// Espace (libre, total) en octets du système de fichiers qui contient `path`.
pub fn disk_space(path: &Path) -> (u64, u64) {
    statvfs(path).map_or((0, 0), |s| (s.f_bavail * s.f_frsize, s.f_blocks * s.f_frsize))
}

/// Nom lisible d'un point de montage : « Système » pour `/`, sinon son dernier composant.
fn disk_name(point: &str) -> String {
    if point == "/" {
        return "Système".to_string();
    }
    Path::new(point).file_name().map_or_else(|| point.to_string(), |n| n.to_string_lossy().into_owned())
}

/// Disques montés (périphériques `/dev/*` hors loop), avec chemin relatif s'ils sont sous la racine.
pub fn mounted_disks(root: &Root) -> Vec<Value> {
    let mut seen: Vec<String> = Vec::new();
    let mut out = Vec::new();
    for mount in read_mounts() {
        let skip = !mount.device.starts_with("/dev/") || mount.device.starts_with("/dev/loop");
        if skip || seen.contains(&mount.device) || IGNORED_POINTS.iter().any(|p| mount.point.starts_with(p)) {
            continue;
        }
        seen.push(mount.device.clone());
        let point = Path::new(&mount.point);
        let (free, total) = disk_space(point);
        let relative = root.inside(point).then(|| root.relative(point));
        out.push(json!({"n": disk_name(&mount.point), "m": mount.point, "dev": mount.device, "fs": mount.fstype,
            "t": total, "f": free, "p": relative}));
    }
    out
}
