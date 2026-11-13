//! Lecture du répertoire central d'une archive zip, sans rien extraire.

use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};

const END_SIGNATURE: [u8; 4] = [0x50, 0x4b, 0x05, 0x06];
const ENTRY_SIGNATURE: [u8; 4] = [0x50, 0x4b, 0x01, 0x02];

/// Une entrée d'archive : nom, taille décompressée, dossier ou non.
pub type Member = (String, u64, bool);

/// Lit un entier little-endian de `width` octets à `at`.
fn le(bytes: &[u8], at: usize, width: usize) -> Option<usize> {
    let slice = bytes.get(at..at + width)?;
    Some(slice.iter().rev().fold(0usize, |acc, byte| (acc << 8) | *byte as usize))
}

/// Indique si le contenu commence comme une archive zip.
pub fn is_zip(head: &[u8]) -> bool {
    head.starts_with(&[0x50, 0x4b, 0x03, 0x04]) || head.starts_with(&END_SIGNATURE)
}

/// Lit la fin du fichier et retourne (nombre d'entrées, taille et position du répertoire central).
fn end_record(file: &mut File) -> io::Result<(usize, usize, u64)> {
    let length = file.metadata()?.len();
    let start = length.saturating_sub(70_000);
    file.seek(SeekFrom::Start(start))?;
    let mut tail = Vec::new();
    file.read_to_end(&mut tail)?;
    let at = tail.windows(4).rposition(|w| w == END_SIGNATURE).ok_or(io::ErrorKind::InvalidData)?;
    let record = |offset, width| le(&tail, at + offset, width).ok_or(io::ErrorKind::InvalidData);
    Ok((record(10, 2)?, record(12, 4)?, record(16, 4)? as u64))
}

/// Entrées d'une archive zip (au plus `limit`) et indicateur d'entrées en plus.
pub fn zip_entries(file: &mut File, limit: usize) -> io::Result<(Vec<Member>, bool)> {
    let (total, size, offset) = end_record(file)?;
    file.seek(SeekFrom::Start(offset))?;
    let mut directory = vec![0u8; size.min(64 << 20)];
    file.read_exact(&mut directory)?;
    let (mut out, mut at) = (Vec::new(), 0);
    while out.len() < limit && directory.get(at..at + 4) == Some(&ENTRY_SIGNATURE) {
        let (name_len, extra, comment) = (le(&directory, at + 28, 2), le(&directory, at + 30, 2), le(&directory, at + 32, 2));
        let (Some(name_len), Some(extra), Some(comment), Some(bytes)) = (name_len, extra, comment, directory.get(at + 46..)) else { break };
        let name = String::from_utf8_lossy(bytes.get(..name_len).ok_or(io::ErrorKind::InvalidData)?).into_owned();
        out.push((name.clone(), le(&directory, at + 24, 4).unwrap_or(0) as u64, name.ends_with('/')));
        at += 46 + name_len + extra + comment;
    }
    Ok((out, total > limit))
}
