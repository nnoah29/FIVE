//! Aiguillage des requêtes GET : ouverture de session, API de lecture, interface web.

use std::collections::HashMap;

use five_engine::refus::{Refus, Resultat};
use tiny_http::Request;

use crate::api_file::{inspect, raw};
use crate::api_folder::{analysis, list};
use crate::api_info::{disks_or_jobs, info, places, view};
use crate::auth::{header_of, require_session};
use crate::reply::{Reply, bytes};
use crate::state::State;
use crate::static_files::static_file;
use crate::token::same_secret;

/// Ouvre la session : vérifie le jeton, pose le cookie et redirige vers la page d'accueil.
fn start_session(state: &State, token: &str) -> Resultat<Reply> {
    if !same_secret(token, &state.token) {
        return Err(Refus::new(401, "Jeton invalide"));
    }
    let cookie = format!("mk={token}; HttpOnly; SameSite=Strict; Path=/");
    Ok(bytes(302, "text/plain", Vec::new(), &[("Set-Cookie", &cookie), ("Location", "/")]))
}

/// Routes qui s'appliquent à un chemin de la racine (`p`).
fn resource(request: &Request, state: &State, name: &str, query: &HashMap<String, String>) -> Resultat<Reply> {
    let full = state.root.resolve(query.get("p").map_or("", String::as_str))?;
    match name {
        "list" => list(state, &full, query),
        "usage" | "duplicates" | "git" => analysis(state, name, &full, query),
        "text" | "archive" => inspect(name, &full),
        "raw" => raw(&full, query, header_of(request, "Range").as_deref()),
        _ => Err(Refus::new(404, "Route inconnue")),
    }
}

/// Route de l'API de lecture `/api/<name>`.
fn api(request: &Request, state: &State, name: &str, query: &HashMap<String, String>) -> Resultat<Reply> {
    match name {
        "places" => Ok(places(state)),
        "info" => Ok(info(state)),
        "disks" | "jobs" => Ok(disks_or_jobs(state, name)),
        "view" => view(state, query),
        _ => resource(request, state, name, query),
    }
}

/// Traite une requête GET.
pub fn handle_get(request: &Request, state: &State, path: &str, query: &HashMap<String, String>) -> Resultat<Reply> {
    if path == "/"
        && let Some(token) = query.get("k")
    {
        return start_session(state, token);
    }
    match path.strip_prefix("/api/") {
        Some(name) => {
            require_session(request, state)?;
            api(request, state, name, query)
        }
        None => static_file(state, path),
    }
}
