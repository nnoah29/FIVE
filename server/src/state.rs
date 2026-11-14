//! État partagé entre les requêtes.

use std::sync::Arc;

use five_engine::jobs::registry::Jobs;
use five_engine::root::Root;

use crate::args::Args;

/// Tout ce dont une requête a besoin : racine, session, transferts, réglages.
pub struct State {
    pub root: Root,
    pub token: String,
    pub hosts: Vec<String>,
    pub jobs: Arc<Jobs>,
    pub port: u16,
    pub user: String,
    pub args: Args,
}

impl State {
    /// Assemble l'état ; les hôtes autorisés sont `127.0.0.1:port` et `localhost:port`.
    pub fn new(root: Root, token: String, args: Args, user: String) -> State {
        let hosts = vec![format!("127.0.0.1:{}", args.port), format!("localhost:{}", args.port)];
        State { root, token, hosts, jobs: Arc::new(Jobs::new()), port: args.port, user, args }
    }
}
