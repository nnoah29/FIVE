//! Types de transfert, états d'avancement et motif d'arrêt.

use std::io;

/// Nature d'un transfert.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Copy,
    Move,
    Zip,
    Unzip,
}

/// État d'un transfert.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Running,
    Paused,
    Done,
    Cancelled,
    Error,
}

/// Raison pour laquelle un transfert s'interrompt.
#[derive(Debug)]
pub enum Stop {
    Cancelled,
    Io(io::Error),
}

impl Kind {
    /// Reconnaît `copy`, `move`, `zip` ou `unzip`.
    pub fn parse(text: &str) -> Option<Kind> {
        match text {
            "copy" => Some(Kind::Copy),
            "move" => Some(Kind::Move),
            "zip" => Some(Kind::Zip),
            "unzip" => Some(Kind::Unzip),
            _ => None,
        }
    }

    /// Nom du type tel que l'API le renvoie.
    pub fn as_str(self) -> &'static str {
        match self {
            Kind::Copy => "copy",
            Kind::Move => "move",
            Kind::Zip => "zip",
            Kind::Unzip => "unzip",
        }
    }
}

impl State {
    /// Nom de l'état tel que l'API le renvoie.
    pub fn as_str(self) -> &'static str {
        match self {
            State::Running => "running",
            State::Paused => "paused",
            State::Done => "done",
            State::Cancelled => "cancelled",
            State::Error => "error",
        }
    }

    /// Vrai tant que le transfert n'est pas terminé.
    pub fn is_live(self) -> bool {
        matches!(self, State::Running | State::Paused)
    }
}

impl From<io::Error> for Stop {
    /// Une erreur système interrompt le transfert.
    fn from(error: io::Error) -> Self {
        Stop::Io(error)
    }
}
