//! Compression d'un fichier dans une archive zip : deflate par blocs, somme de contrôle, limite de taille.

use std::fs::File;
use std::io::{self, ErrorKind, Read, Write};

use flate2::write::DeflateEncoder;
use flate2::{Compression, Crc};

use crate::jobs::job::Job;
use crate::jobs::state::Stop;

const CHUNK: usize = 1 << 20;

/// Convertit une taille en `u32`, ou refuse ce qui dépasse la limite d'une archive zip standard.
pub fn small(value: u64) -> io::Result<u32> {
    u32::try_from(value).map_err(|_| ErrorKind::FileTooLarge.into())
}

/// Lit `reader` par blocs en le compressant dans `writer` ; retourne la somme de contrôle et la taille d'origine.
pub fn deflate_stream(job: &Job, reader: &mut File, writer: &mut File) -> Result<(u32, u64), Stop> {
    let mut encoder = DeflateEncoder::new(writer, Compression::default());
    let (mut crc, mut block, mut size) = (Crc::new(), vec![0u8; CHUNK], 0u64);
    loop {
        job.checkpoint()?;
        let read = reader.read(&mut block)?;
        if read == 0 {
            break;
        }
        size += small(read as u64)? as u64;
        crc.update(&block[..read]);
        encoder.write_all(&block[..read])?;
        job.update(|p| p.done += read as u64);
    }
    encoder.finish()?;
    Ok((crc.sum(), small(size)? as u64))
}
