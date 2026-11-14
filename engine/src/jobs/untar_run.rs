//! Extraction d'une archive tar (éventuellement compressée) : dossiers et fichiers ordinaires seulement.

use std::fs::{self, OpenOptions};
use std::io::{self, ErrorKind, Read};
use std::path::Path;

use super::extract_copy::{copy_limited, set_mode};
use super::job::Job;
use super::state::Stop;
use super::unzip_run::MAX_TOTAL;
use crate::archive_path::safe_join;
use crate::tar_header::{BLOCK, entry_name, next_header, octal, take_data, text};

const MAX_LONG_NAME: u64 = 1 << 20;

/// Écrit un fichier de `size` octets lus dans `reader`, puis saute le remplissage du bloc.
fn write_entry(job: &Job, reader: &mut impl Read, target: &Path, size: u64, mode: u32) -> Result<(), Stop> {
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut out = OpenOptions::new().write(true).create(true).truncate(true).open(target)?;
    let (_, written) = copy_limited(job, &mut reader.take(size), &mut out, size)?;
    if written < size {
        return Err(Stop::Io(ErrorKind::UnexpectedEof.into()));
    }
    let padding = size.div_ceil(BLOCK as u64) * BLOCK as u64 - size;
    io::copy(&mut reader.take(padding), &mut io::sink())?;
    Ok(set_mode(target, mode)?)
}

/// Traite une entrée : dossier créé, fichier ordinaire écrit, tout le reste (liens, périphériques) ignoré.
fn extract_entry(job: &Job, reader: &mut impl Read, header: &[u8; BLOCK], name: &str, folder: &Path) -> Result<(), Stop> {
    let (size, kind) = (octal(&header[124..136]), header[156]);
    let Some(target) = safe_join(folder, name)? else { return Ok(take_data(reader, size, false).map(drop)?) };
    match kind {
        b'5' => Ok(fs::create_dir_all(&target)?),
        0 | b'0' | b'7' => write_entry(job, reader, &target, size, octal(&header[100..108]) as u32),
        _ => Ok(take_data(reader, size, false).map(drop)?),
    }
}

/// Extrait un flux tar dans `folder` ; les noms longs sont bornés et la taille cumulée plafonnée à 100 Go.
pub fn extract_tar(job: &Job, mut reader: impl Read, folder: &Path) -> Result<(), Stop> {
    let (mut long_name, mut total, mut first) = (None::<String>, 0u64, true);
    while let Some(header) = next_header(&mut reader, first)? {
        job.checkpoint()?;
        first = false;
        let (size, kind) = (octal(&header[124..136]), header[156]);
        if matches!(kind, b'L' | b'x' | b'g') {
            if size > MAX_LONG_NAME {
                return Err(Stop::Io(ErrorKind::InvalidData.into()));
            }
            let data = take_data(&mut reader, size, true)?;
            long_name = (kind == b'L').then(|| text(&data));
            continue;
        }
        total += size;
        if total > MAX_TOTAL {
            return Err(Stop::Io(ErrorKind::FileTooLarge.into()));
        }
        extract_entry(job, &mut reader, &header, &entry_name(&header, long_name.take()), folder)?;
    }
    Ok(())
}
