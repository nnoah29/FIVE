//! Transferts de fichiers en arrière-plan : copie, déplacement, compression, extraction, pause, annulation.

pub mod compress;
pub mod copy;
pub mod counted;
pub mod extract;
pub mod extract_copy;
pub mod job;
pub mod measure;
pub mod registry;
pub mod start;
pub mod state;
pub mod transfer;
pub mod untar_run;
pub mod unzip_run;
