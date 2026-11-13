//! Opérations d'écriture : dossier, suppression, renommage, transferts, déconnexion.

use five_engine::delete::delete_entry;
use five_engine::folders::make_folder;
use five_engine::jobs::start::{control_job, start_job};
use five_engine::refus::{Refus, Resultat};
use five_engine::rename::rename_many;
use serde_json::{Value, json};

use crate::body::{text, texts};
use crate::reply::{Reply, json_reply, with_headers};
use crate::state::State;

/// `/api/mkdir` : crée un dossier.
pub fn mkdir(state: &State, body: &Value) -> Resultat<Reply> {
    let made = make_folder(&state.root, text(body, "p"), text(body, "name"))?;
    Ok(json_reply(201, &json!({"p": made})))
}

/// `/api/delete` : corbeille, ou effacement définitif depuis la corbeille.
pub fn delete(state: &State, body: &Value) -> Resultat<Reply> {
    let paths = texts(body, "paths");
    if paths.is_empty() || paths.len() > 100 {
        return Err(Refus::new(400, "Liste de chemins invalide"));
    }
    let modes = paths.iter().map(|p| delete_entry(&state.root, p)).collect::<Resultat<Vec<_>>>()?;
    Ok(json_reply(200, &json!({"mode": modes[0].as_str(), "count": modes.len()})))
}

/// `/api/rename` : renommage par lot.
pub fn rename(state: &State, body: &Value) -> Resultat<Reply> {
    let items = body.get("items").and_then(Value::as_array).map_or(&[][..], Vec::as_slice);
    let pairs: Vec<(String, String)> = items.iter().map(|i| (text(i, "p").to_string(), text(i, "name").to_string())).collect();
    if pairs.is_empty() || pairs.len() > 500 {
        return Err(Refus::new(400, "Liste de renommages invalide"));
    }
    Ok(json_reply(200, &json!({"count": rename_many(&state.root, &pairs)?})))
}

/// `/api/job` et `/api/job/control` : démarre un transfert ou le contrôle.
pub fn job(state: &State, body: &Value, control: bool) -> Resultat<Reply> {
    if control {
        control_job(&state.jobs, text(body, "id"), text(body, "action"))?;
        return Ok(json_reply(200, &json!({"ok": true})));
    }
    let mut paths = texts(body, "paths");
    paths.truncate(500);
    if paths.is_empty() {
        return Err(Refus::new(400, "Aucun élément"));
    }
    let id = start_job(&state.root, &state.jobs, text(body, "kind"), &paths, text(body, "dest"))?;
    Ok(json_reply(201, &json!({"id": id})))
}

/// `/api/logout` : efface le cookie de session.
pub fn logout() -> Reply {
    with_headers(200, &json!({"ok": true}), &[("Set-Cookie", "mk=; HttpOnly; SameSite=Strict; Path=/; Max-Age=0")])
}
