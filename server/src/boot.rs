//! Démarrage : racine, jeton, état partagé, ouverture du navigateur.

use std::path::PathBuf;
use std::process::Command;

use five_engine::root::Root;

use crate::args::Args;
use crate::state::State;
use crate::token::new_token;

/// Remplace un `~` initial par le dossier personnel.
fn expand_home(text: &str, home: &str) -> PathBuf {
    match text.strip_prefix('~') {
        Some(rest) => PathBuf::from(format!("{home}{rest}")),
        None => PathBuf::from(text),
    }
}

/// Construit l'état partagé depuis les arguments et l'environnement.
pub fn build_state(args: Args) -> Result<State, String> {
    let home = std::env::var("HOME").map_err(|_| "HOME n'est pas défini")?;
    let root = Root::new(&expand_home(&args.root, &home), &PathBuf::from(&home)).map_err(|e| format!("Racine illisible : {e}"))?;
    let token = new_token().map_err(|e| format!("Jeton impossible : {e}"))?;
    let user = std::env::var("USER").or_else(|_| std::env::var("LOGNAME")).unwrap_or_default();
    Ok(State::new(root, token, args, user))
}

/// Adresse de lancement, avec le jeton de session.
pub fn launch_url(state: &State) -> String {
    format!("http://127.0.0.1:{}/?k={}", state.port, state.token)
}

/// Ouvre l'adresse dans le navigateur par défaut ; l'échec est sans conséquence.
pub fn open_browser(url: &str) {
    let _ = Command::new("xdg-open").arg(url).spawn();
}
