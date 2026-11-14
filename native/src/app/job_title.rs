//! Intitulé d'un transfert dans la file : nature et nombre d'éléments.

use serde_json::Value;

/// Titre d'un transfert : sa nature et le nombre d'éléments (ou d'archives) concernés.
pub fn job_title(job: &Value) -> String {
    let (kind, noun) = match job["kind"].as_str() {
        Some("copy") => ("Copie", "élément(s)"),
        Some("zip") => ("Compression", "élément(s)"),
        Some("unzip") => ("Extraction", "archive(s)"),
        _ => ("Déplacement", "élément(s)"),
    };
    format!("{kind} de {} {noun}", job["count"])
}
