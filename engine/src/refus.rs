//! Refus HTTP du moteur : un code et un message en français.

use std::fmt;
use std::io::{Error, ErrorKind};

/// Erreur métier portant le code HTTP à renvoyer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Refus {
    pub code: u16,
    pub message: String,
}

/// Résultat des opérations du moteur.
pub type Resultat<T> = Result<T, Refus>;

impl Refus {
    /// Construit un refus avec son code HTTP et son message.
    pub fn new(code: u16, message: impl Into<String>) -> Self {
        Self { code, message: message.into() }
    }
}

impl fmt::Display for Refus {
    /// Affiche le message du refus.
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        out.write_str(&self.message)
    }
}

impl From<Error> for Refus {
    /// Traduit une erreur système en refus HTTP.
    fn from(error: Error) -> Self {
        match error.kind() {
            ErrorKind::PermissionDenied => Refus::new(403, "Accès refusé par le système"),
            ErrorKind::NotFound | ErrorKind::NotADirectory => Refus::new(404, "Élément introuvable"),
            ErrorKind::AlreadyExists => Refus::new(409, "Un élément de ce nom existe déjà"),
            _ => Refus::new(500, "Erreur système"),
        }
    }
}
