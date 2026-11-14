//! Copie bornée vers un fichier extrait : somme de contrôle, point d'arrêt, taille maximale.

use std::fs::{self, File, Permissions};
use std::io::{self, ErrorKind, Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use flate2::Crc;

use super::job::Job;
use super::state::Stop;

const CHUNK: usize = 1 << 20;

/// Copie `reader` dans `writer` par blocs ; refuse d'écrire plus de `limit` octets (archive piégée ou abîmée).
pub fn copy_limited(job: &Job, reader: &mut dyn Read, writer: &mut File, limit: u64) -> Result<(u32, u64), Stop> {
    let (mut crc, mut block, mut size) = (Crc::new(), vec![0u8; CHUNK], 0u64);
    loop {
        job.checkpoint()?;
        let read = reader.read(&mut block)?;
        if read == 0 {
            return Ok((crc.sum(), size));
        }
        size += read as u64;
        if size > limit {
            return Err(Stop::Io(io::Error::from(ErrorKind::InvalidData)));
        }
        crc.update(&block[..read]);
        writer.write_all(&block[..read])?;
    }
}

/// Applique les droits de l'archive (lecture et écriture du propriétaire toujours conservées, jamais de setuid) ; 0 donne 644.
pub fn set_mode(path: &Path, mode: u32) -> io::Result<()> {
    let bits = if mode & 0o777 == 0 { 0o644 } else { (mode & 0o777) | 0o600 };
    fs::set_permissions(path, Permissions::from_mode(bits))
}
