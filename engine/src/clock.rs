//! Horloge locale : le décalage horaire est lu une fois, avant le démarrage des threads.

use std::sync::OnceLock;

use time::{OffsetDateTime, UtcOffset};

static OFFSET: OnceLock<UtcOffset> = OnceLock::new();

/// Lit le décalage horaire local ; à appeler au démarrage, avant tout thread.
pub fn init_offset() {
    let offset = UtcOffset::current_local_offset().unwrap_or(UtcOffset::UTC);
    let _ = OFFSET.set(offset);
}

/// Date et heure locales au format `AAAA-MM-JJThh:mm:ss`.
pub fn local_stamp() -> String {
    let offset = OFFSET.get().copied().unwrap_or(UtcOffset::UTC);
    let now = OffsetDateTime::now_utc().to_offset(offset);
    format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}", now.year(), u8::from(now.month()), now.day(), now.hour(), now.minute(), now.second())
}

/// Champs locaux (année, mois, jour, heure, minute) d'un instant en secondes Unix.
pub fn local_fields(seconds: f64) -> (i32, u8, u8, u8, u8) {
    let offset = OFFSET.get().copied().unwrap_or(UtcOffset::UTC);
    let moment = OffsetDateTime::from_unix_timestamp(seconds as i64).unwrap_or(OffsetDateTime::UNIX_EPOCH).to_offset(offset);
    (moment.year(), u8::from(moment.month()), moment.day(), moment.hour(), moment.minute())
}

/// Secondes Unix de l'instant présent.
pub fn now_seconds() -> f64 {
    OffsetDateTime::now_utc().unix_timestamp() as f64
}
