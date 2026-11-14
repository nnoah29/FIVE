//! Traitement d'une requête : contrôle de l'hôte, aiguillage, envoi de la réponse.

use five_engine::refus::{Refus, Resultat};
use tiny_http::{Method, Request};

use crate::auth::check_host;
use crate::query::{parse_query, split_url};
use crate::reply::{Reply, refus_reply};
use crate::routes_get::handle_get;
use crate::routes_post::handle_post;
use crate::routes_rules::handle_rules;
use crate::state::State;

/// Choisit le traitement selon la méthode HTTP.
fn dispatch(request: &mut Request, state: &State) -> Resultat<Reply> {
    check_host(request, state)?;
    let url = request.url().to_string();
    let (path, raw_query) = split_url(&url);
    let method = request.method().clone();
    if path == "/api/rules" || path.starts_with("/api/rules/") {
        return handle_rules(request, state, &method, path, &parse_query(raw_query));
    }
    match method {
        Method::Get => handle_get(request, state, path, &parse_query(raw_query)),
        Method::Post => handle_post(request, state, path),
        _ => Err(Refus::new(405, "Méthode non prise en charge")),
    }
}

/// Traite une requête et envoie la réponse ; une coupure de connexion est ignorée.
pub fn serve(mut request: Request, state: &State) {
    let reply = dispatch(&mut request, state).unwrap_or_else(|refus| refus_reply(&refus));
    let _ = request.respond(reply);
}
