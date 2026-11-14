//! Calculs de dates des règles : durées en secondes, jour, mois, année locaux, mise en forme `AAAA-MM-JJ`.

use crate::clock::{local_fields, local_weekday};

use super::types_matcher::{DurationSpec, Unit, Weekday};

/// Durée en secondes (un mois vaut 30 jours).
pub fn seconds(duration: &DurationSpec) -> f64 {
    let unit = match duration.unit {
        Unit::Seconds => 1.0,
        Unit::Minutes => 60.0,
        Unit::Hours => 3_600.0,
        Unit::Days => 86_400.0,
        Unit::Weeks => 604_800.0,
        Unit::Months => 2_592_000.0,
    };
    duration.n as f64 * unit
}

/// Jour de la semaine local d'un instant.
pub fn weekday(at: f64) -> Weekday {
    [Weekday::Mon, Weekday::Tue, Weekday::Wed, Weekday::Thu, Weekday::Fri, Weekday::Sat, Weekday::Sun][local_weekday(at) as usize % 7]
}

/// Année, mois (1 à 12) et jour locaux d'un instant.
pub fn civil(at: f64) -> (i32, u8, u8) {
    let (year, month, day, _, _) = local_fields(at);
    (year, month, day)
}

/// Met un instant en forme : `AAAA`, `AA`, `MM`, `JJ`, `hh`, `mm` sont remplacés ; tout le reste est conservé.
pub fn format_date(at: f64, spec: &str) -> String {
    let (year, month, day, hour, minute) = local_fields(at);
    let tokens = [
        ("AAAA", format!("{year:04}")),
        ("AA", format!("{:02}", year % 100)),
        ("MM", format!("{month:02}")),
        ("JJ", format!("{day:02}")),
        ("hh", format!("{hour:02}")),
        ("mm", format!("{minute:02}")),
    ];
    let mut out = String::new();
    let mut rest = spec;
    while !rest.is_empty() {
        match tokens.iter().find(|(token, _)| rest.starts_with(token)) {
            Some((token, value)) => {
                out.push_str(value);
                rest = &rest[token.len()..];
            }
            None => {
                let next = rest.chars().next().map_or(1, char::len_utf8);
                out.push_str(&rest[..next]);
                rest = &rest[next..];
            }
        }
    }
    out
}

/// Lit `AAAA-MM-JJ hh:mm` (ou `AAAA-MM-JJ`) comme une date locale et la rend en secondes Unix.
pub fn parse_local(text: &str) -> Option<f64> {
    let (date, clock) = text.trim().split_once(' ').unwrap_or((text.trim(), "00:00"));
    let number = |part: Option<&str>| part.and_then(|p| p.parse::<i32>().ok());
    let mut d = date.split('-');
    let mut c = clock.split(':');
    let (y, m, day) = (number(d.next())?, number(d.next())?, number(d.next())?);
    crate::clock::local_timestamp(y, m as u8, day as u8, number(c.next())? as u8, number(c.next())? as u8)
}
