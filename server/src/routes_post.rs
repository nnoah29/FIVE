//! Aiguillage des requêtes POST : contrôles d'écriture, corps JSON, opérations.

use std::io::Read;

use five_engine::refus::{Refus, Resultat};
use serde_json::Value;
use tiny_http::Request;

use crate::api_write::{delete, job, logout, mkdir, rename};
use crate::auth::check_write;
use crate::reply::Reply;
use crate::state::State;

const MAX_BODY: usize = 65_536;

/// Lit le corps de la requête : JSON de type objet, 64 Ko au plus.
fn read_body(request: &mut Request) -> Resultat<Value> {
    if request.body_length().is_some_and(|length| length > MAX_BODY) {
        return Err(Refus::new(413, "Requête trop volumineuse"));
    }
    let mut raw = Vec::new();
    request.as_reader().take(MAX_BODY as u64 + 1).read_to_end(&mut raw).map_err(|_| Refus::new(400, "Corps illisible"))?;
    if raw.len() > MAX_BODY {
        return Err(Refus::new(413, "Requête trop volumineuse"));
    }
    let body: Value =
        if raw.is_empty() { Value::Object(Default::default()) } else { serde_json::from_slice(&raw).map_err(|_| Refus::new(400, "Corps JSON invalide"))? };
    if body.is_object() { Ok(body) } else { Err(Refus::new(400, "Corps JSON invalide")) }
}

/// Traite une requête POST.
pub fn handle_post(request: &mut Request, state: &State, path: &str) -> Resultat<Reply> {
    check_write(request, state)?;
    let body = read_body(request)?;
    match path {
        "/api/mkdir" => mkdir(state, &body),
        "/api/delete" => delete(state, &body),
        "/api/rename" => rename(state, &body),
        "/api/job" => job(state, &body, false),
        "/api/job/control" => job(state, &body, true),
        "/api/logout" => Ok(logout()),
        _ => Err(Refus::new(404, "Route inconnue")),
    }
}
