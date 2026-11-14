//! Lecteur qui compte les octets lus de l'archive pour faire avancer la progression du transfert.

use std::io::{self, Read};

use super::job::Job;

/// Enveloppe un lecteur et reporte chaque lecture dans la progression du transfert.
pub struct Counted<'a, R> {
    pub inner: R,
    pub job: &'a Job,
}

impl<R: Read> Read for Counted<'_, R> {
    /// Lit des octets et les ajoute à la progression.
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        let read = self.inner.read(buffer)?;
        self.job.update(|p| p.done += read as u64);
        Ok(read)
    }
}
