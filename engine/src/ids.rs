//! Identifiants uniques courts, pour les noms temporaires et les transferts.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static COUNTER: AtomicU64 = AtomicU64::new(0);

/// Identifiant hexadécimal de 16 caractères, unique dans ce processus et très probablement ailleurs.
pub fn unique_id() -> String {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.subsec_nanos() as u64);
    let seconds = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs());
    let count = COUNTER.fetch_add(1, Ordering::SeqCst);
    format!("{:08x}{:08x}", (seconds << 20 ^ nanos) as u32, (count << 16 ^ std::process::id() as u64) as u32)
}
