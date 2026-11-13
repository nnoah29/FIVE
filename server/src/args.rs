//! Arguments de la ligne de commande.

use std::path::PathBuf;

/// Options de lancement du serveur.
#[derive(Debug, Clone)]
pub struct Args {
    pub root: String,
    pub port: u16,
    pub open: bool,
    pub web: Option<PathBuf>,
}

impl Args {
    /// Lit les arguments ; `Err` contient le message d'aide ou d'erreur à afficher.
    pub fn parse(mut input: impl Iterator<Item = String>) -> Result<Args, String> {
        let mut args = Args { root: "~".to_string(), port: 8765, open: false, web: None };
        while let Some(flag) = input.next() {
            match flag.as_str() {
                "--root" => args.root = input.next().ok_or("--root attend un dossier")?,
                "--port" => args.port = input.next().and_then(|v| v.parse().ok()).ok_or("--port attend un numéro")?,
                "--web" => args.web = Some(input.next().ok_or("--web attend un dossier")?.into()),
                "--open" => args.open = true,
                _ => return Err(USAGE.to_string()),
            }
        }
        Ok(args)
    }
}

const USAGE: &str = "five [--root DOSSIER] [--port N] [--open] [--web DOSSIER]\n  --root  dossier exposé (défaut : ~)\n  --port  port local (défaut : 8765)\n  --open  ouvre le navigateur\n  --web   sert l'interface depuis ce dossier au lieu de la version embarquée";
