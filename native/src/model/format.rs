//! Mise en forme à la française : tailles, nombres, dates, durées.

use five_engine::clock::{local_fields, now_seconds};

use super::numbers::group_text;

const MONTHS: [&str; 12] = ["janv.", "févr.", "mars", "avr.", "mai", "juin", "juil.", "août", "sept.", "oct.", "nov.", "déc."];

/// Taille lisible : `1,5 Mo` (unités SI) ou `1,4 Mio` (unités binaires).
pub fn format_size(bytes: u64, binary: bool) -> String {
    let base = if binary { 1024.0 } else { 1000.0 };
    if (bytes as f64) < base {
        return format!("{bytes} o");
    }
    let units = if binary { ["Kio", "Mio", "Gio", "Tio"] } else { ["Ko", "Mo", "Go", "To"] };
    let (mut value, mut at) = (bytes as f64, 0);
    loop {
        value /= base;
        if value < base || at == 3 {
            break;
        }
        at += 1;
    }
    let text = if value < 10.0 { format!("{:.1}", (value * 10.0).round() / 10.0) } else { format!("{:.0}", value) };
    let text = text.trim_end_matches('0').trim_end_matches('.').replace('.', ",");
    format!("{} {}", group_text(&text), units[at])
}

/// Heure locale `hh:mm`.
pub fn format_time(seconds: f64) -> String {
    let (_, _, _, hour, minute) = local_fields(seconds);
    format!("{hour:02}:{minute:02}")
}

/// Jour local `27 oct. 2026`.
pub fn format_day(seconds: f64) -> String {
    let (year, month, day, _, _) = local_fields(seconds);
    format!("{day} {} {year}", MONTHS[(month as usize).clamp(1, 12) - 1])
}

/// Date d'un élément : l'heure pour aujourd'hui, sinon le jour ; `full` ajoute toujours l'heure.
pub fn format_date(seconds: f64, full: bool) -> String {
    if full {
        return format!("{} {}", format_day(seconds), format_time(seconds));
    }
    let same_day = local_fields(seconds).0 == local_fields(now_seconds()).0 && format_day(seconds) == format_day(now_seconds());
    if same_day { format_time(seconds) } else { format_day(seconds) }
}

/// Durée `m:ss` ou `h:mm:ss`, `—` si elle est inconnue.
pub fn format_clock(seconds: f64) -> String {
    if !seconds.is_finite() {
        return "—".to_string();
    }
    let total = seconds.round() as u64;
    let (hours, minutes, rest) = (total / 3600, total % 3600 / 60, total % 60);
    if hours > 0 { format!("{hours}:{minutes:02}:{rest:02}") } else { format!("{minutes}:{rest:02}") }
}
