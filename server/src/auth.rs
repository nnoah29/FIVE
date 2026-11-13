//! Contrôles d'accès : hôte, cookie de session, origine des écritures.

use five_engine::refus::{Refus, Resultat};
use tiny_http::Request;

use crate::state::State;
use crate::token::same_secret;

/// Valeur d'un en-tête de la requête, sans tenir compte de la casse du nom.
pub fn header_of(request: &Request, name: &str) -> Option<String> {
    let found = request.headers().iter().find(|h| h.field.as_str().as_str().eq_ignore_ascii_case(name));
    found.map(|h| h.value.as_str().to_string())
}

/// Refuse toute requête dont l'en-tête `Host` n'est pas local (anti-DNS-rebinding).
pub fn check_host(request: &Request, state: &State) -> Resultat<()> {
    let host = header_of(request, "Host").unwrap_or_default();
    if state.hosts.contains(&host) { Ok(()) } else { Err(Refus::new(403, "Hôte non autorisé")) }
}

/// Indique si le cookie `mk` porte le jeton de session.
pub fn authorised(request: &Request, state: &State) -> bool {
    let cookies = header_of(request, "Cookie").unwrap_or_default();
    cookies.split(';').filter_map(|c| c.trim().strip_prefix("mk=")).any(|value| same_secret(value, &state.token))
}

/// Exige la session pour les routes `/api/`.
pub fn require_session(request: &Request, state: &State) -> Resultat<()> {
    if authorised(request, state) { Ok(()) } else { Err(Refus::new(401, "Session absente : ouvrez l'adresse affichée dans le terminal")) }
}

/// Contrôles propres aux écritures : en-tête `X-Five`, origine locale, session.
pub fn check_write(request: &Request, state: &State) -> Resultat<()> {
    let origin = header_of(request, "Origin");
    let local = origin.as_deref().is_none_or(|o| state.hosts.iter().any(|h| o.split("://").nth(1) == Some(h.as_str())));
    if header_of(request, "X-Five").as_deref() != Some("1") || !local {
        return Err(Refus::new(403, "Requête refusée"));
    }
    require_session(request, state)
}
