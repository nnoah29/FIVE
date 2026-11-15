//! Commandes `--enable-chooser`, `--disable-chooser` et `--chooser-status`.

use std::path::PathBuf;

use gtk4::glib::ExitCode;

use super::portal_enable::{chooser_enabled, set_enabled};

/// Exécute la commande demandée ; `None` si ce n'en est pas une.
pub fn run_if_requested(flag: &str) -> Option<ExitCode> {
    let home = std::env::var_os("HOME").map(PathBuf::from)?;
    let enabled = match flag {
        "--chooser-status" => {
            println!("{}", if chooser_enabled(&home) { "FIVE est votre sélecteur de fichiers." } else { "FIVE n'est pas votre sélecteur de fichiers." });
            return Some(ExitCode::SUCCESS);
        }
        "--enable-chooser" => true,
        "--disable-chooser" => false,
        _ => return None,
    };
    Some(match set_enabled(&home, enabled) {
        Ok(restarted) => {
            println!("{}", if enabled { "FIVE sera votre sélecteur de fichiers." } else { "Le sélecteur de fichiers par défaut est rétabli." });
            if !restarted {
                println!("Relancez le portail pour appliquer : systemctl --user restart xdg-desktop-portal");
            }
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("Impossible de modifier la configuration : {error}");
            ExitCode::FAILURE
        }
    })
}
