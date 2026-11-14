//! FIVE : serveur local qui expose les fichiers de l'utilisateur à l'interface web.

mod api_file;
mod api_folder;
mod api_info;
mod api_rules;
mod api_rules_run;
mod api_write;
mod args;
mod auth;
mod body;
mod boot;
mod file_send;
mod http;
mod query;
mod reply;
mod routes_get;
mod routes_post;
mod routes_rules;
mod state;
mod static_files;
mod token;

use std::process::ExitCode;
use std::sync::Arc;

use tiny_http::Server;

/// Point d'entrée : lit le décalage horaire avant tout thread, puis lance le serveur.
fn main() -> ExitCode {
    five_engine::clock::init_offset();
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::from(2)
        }
    }
}

/// Démarre le serveur sur 127.0.0.1 et traite chaque requête dans son propre thread.
fn run() -> Result<(), String> {
    let args = args::Args::parse(std::env::args().skip(1))?;
    let state = Arc::new(boot::build_state(args)?);
    let server = Server::http(format!("127.0.0.1:{}", state.port)).map_err(|e| format!("Écoute impossible : {e}"))?;
    let url = boot::launch_url(&state);
    println!("FIVE — racine : {}\nOuvrez : {url}", state.root.path().display());
    if state.args.open {
        boot::open_browser(&url);
    }
    for request in server.incoming_requests() {
        let state = Arc::clone(&state);
        std::thread::spawn(move || http::serve(request, &state));
    }
    Ok(())
}
