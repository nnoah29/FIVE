//! Fabrique de réponses HTTP.

use std::io::{Cursor, Read};

use five_engine::refus::Refus;
use serde_json::{Value, json};
use tiny_http::{Header, Response};

/// Réponse HTTP prête à être envoyée.
pub type Reply = Response<Box<dyn Read + Send>>;

/// Construit la liste d'en-têtes ; une valeur invalide est simplement omise.
pub fn headers(pairs: &[(&str, &str)]) -> Vec<Header> {
    pairs.iter().filter_map(|(name, value)| Header::from_bytes(name.as_bytes(), value.as_bytes()).ok()).collect()
}

/// Réponse avec corps en mémoire et en-têtes.
pub fn bytes(code: u16, kind: &str, body: Vec<u8>, extra: &[(&str, &str)]) -> Reply {
    let length = body.len();
    let mut all = headers(&[("Content-Type", kind)]);
    all.extend(headers(extra));
    let reader: Box<dyn Read + Send> = Box::new(Cursor::new(body));
    Response::new(code.into(), all, reader, Some(length), None)
}

/// Réponse JSON, jamais mise en cache.
pub fn json_reply(code: u16, payload: &Value) -> Reply {
    let body = payload.to_string().into_bytes();
    bytes(code, "application/json; charset=utf-8", body, &[("Cache-Control", "no-store")])
}

/// Réponse JSON d'erreur `{"erreur": message}`.
pub fn refus_reply(refus: &Refus) -> Reply {
    json_reply(refus.code, &json!({"erreur": refus.message}))
}

/// Réponse JSON avec en-têtes supplémentaires (cookie, redirection).
pub fn with_headers(code: u16, payload: &Value, extra: &[(&str, &str)]) -> Reply {
    let mut reply = json_reply(code, payload);
    for header in headers(extra) {
        reply.add_header(header);
    }
    reply
}
