//! Aiguillage de `/api/rules` : lectures avec session, écritures avec en-tête `X-Five` et origine locale.

use std::collections::HashMap;

use five_engine::refus::{Refus, Resultat};
use tiny_http::{Method, Request};

use crate::api_rules::{rule_change, rule_create, rules_catalog_reply, rules_history_reply, rules_list};
use crate::api_rules_run::rules_action;
use crate::auth::{check_write, require_session};
use crate::reply::Reply;
use crate::routes_post::read_body;
use crate::state::State;

/// Indique si le texte peut être un identifiant de règle.
fn is_rule_id(id: &str) -> bool {
    !id.is_empty() && id.len() <= 64 && id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

/// Lectures : liste, catalogue, journal.
fn read(state: &State, sub: &str, query: &HashMap<String, String>) -> Resultat<Reply> {
    match sub {
        "" => Ok(rules_list(state)),
        "catalog" => Ok(rules_catalog_reply()),
        "history" => Ok(rules_history_reply(state, query)),
        _ => Err(Refus::new(404, "Route inconnue")),
    }
}

/// Écritures : création, modification, suppression, actions.
fn write(request: &mut Request, state: &State, method: &Method, sub: &str) -> Resultat<Reply> {
    check_write(request, state)?;
    let body = read_body(request)?;
    match (method, sub) {
        (Method::Post, "") => rule_create(state, &body),
        (Method::Post, action) => rules_action(state, action, &body),
        (Method::Put, id) if is_rule_id(id) => rule_change(state, id, Some(&body)),
        (Method::Delete, id) if is_rule_id(id) => rule_change(state, id, None),
        _ => Err(Refus::new(404, "Route inconnue")),
    }
}

/// Traite une requête `/api/rules[/…]`.
pub fn handle_rules(request: &mut Request, state: &State, method: &Method, path: &str, query: &HashMap<String, String>) -> Resultat<Reply> {
    let sub = path.strip_prefix("/api/rules").unwrap_or("").trim_start_matches('/');
    match method {
        Method::Get => {
            require_session(request, state)?;
            read(state, sub, query)
        }
        Method::Post | Method::Put | Method::Delete => write(request, state, method, sub),
        _ => Err(Refus::new(405, "Méthode non prise en charge")),
    }
}
