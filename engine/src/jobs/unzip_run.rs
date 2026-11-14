//! Extraction d'une archive zip : chaque entrée est vérifiée (nom, taille annoncée, somme de contrôle).

use std::fs::{self, File, OpenOptions};
use std::io::{ErrorKind, Read, Seek, SeekFrom};
use std::path::Path;

use flate2::read::DeflateDecoder;

use super::extract_copy::{copy_limited, set_mode};
use super::job::Job;
use super::state::Stop;
use crate::archive_path::safe_join;
use crate::zip_read::{ZipMember, data_start, zip_members};

pub const MAX_TOTAL: u64 = 100 << 30;

/// Refuse une archive dont la taille annoncée une fois extraite dépasse 100 Go.
fn check_total(members: &[ZipMember]) -> Result<(), Stop> {
    let total: u64 = members.iter().map(|m| m.size).sum();
    if total > MAX_TOTAL { Err(Stop::Io(ErrorKind::FileTooLarge.into())) } else { Ok(()) }
}

/// Écrit une entrée : décompression, puis contrôle de la taille et de la somme de contrôle annoncées.
fn write_member(job: &Job, member: &ZipMember, reader: impl Read, target: &Path) -> Result<(), Stop> {
    let mut out = OpenOptions::new().write(true).create(true).truncate(true).open(target)?;
    let mut source: Box<dyn Read> = if member.method == 8 { Box::new(DeflateDecoder::new(reader)) } else { Box::new(reader) };
    let (crc, size) = copy_limited(job, &mut *source, &mut out, member.size)?;
    if crc != member.crc || size != member.size {
        return Err(Stop::Io(ErrorKind::InvalidData.into()));
    }
    Ok(set_mode(target, member.mode)?)
}

/// Extrait une entrée sous `folder` ; un dossier est créé, un chiffrement ou une méthode inconnue sont refusés.
fn extract_member(job: &Job, file: &mut File, member: &ZipMember, folder: &Path) -> Result<(), Stop> {
    let Some(target) = safe_join(folder, &member.name)? else { return Ok(()) };
    if member.name.ends_with('/') {
        return Ok(fs::create_dir_all(&target)?);
    }
    if member.flags & 1 != 0 || !matches!(member.method, 0 | 8) {
        return Err(Stop::Io(ErrorKind::Unsupported.into()));
    }
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
    }
    let start = data_start(file, member)?;
    file.seek(SeekFrom::Start(start))?;
    write_member(job, member, file.take(member.csize), &target)
}

/// Extrait une archive zip dans `folder` ; la progression suit les octets compressés lus.
pub fn extract_zip(job: &Job, archive: &Path, folder: &Path) -> Result<(), Stop> {
    let mut file = File::open(archive)?;
    let members = zip_members(&mut file)?;
    check_total(&members)?;
    for member in &members {
        job.checkpoint()?;
        extract_member(job, &mut file, member, folder)?;
        job.update(|p| p.done += member.csize);
    }
    Ok(())
}
