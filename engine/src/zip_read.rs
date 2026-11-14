//! Répertoire central complet d'une archive zip, avec ce qu'il faut pour extraire chaque entrée.

use std::fs::File;
use std::io::{self, ErrorKind, Read, Seek, SeekFrom};

use crate::zip::{end_record, le};

const ENTRY_SIGNATURE: [u8; 4] = [0x50, 0x4b, 0x01, 0x02];
const LOCAL_SIGNATURE: [u8; 4] = [0x50, 0x4b, 0x03, 0x04];
const MAX_MEMBERS: usize = 100_000;

/// Entrée du répertoire central d'une archive à extraire.
#[derive(Debug, Clone)]
pub struct ZipMember {
    pub name: String,
    pub method: u16,
    pub flags: u16,
    pub crc: u32,
    pub csize: u64,
    pub size: u64,
    pub offset: u64,
    pub mode: u32,
}

/// Lit une entrée à la position `at` du répertoire ; retourne l'entrée et la position de la suivante.
fn parse_member(directory: &[u8], at: usize) -> io::Result<(ZipMember, usize)> {
    let bad = || io::Error::from(ErrorKind::InvalidData);
    let field = |offset: usize, width: usize| le(directory, at + offset, width).ok_or_else(bad);
    if directory.get(at..at + 4) != Some(&ENTRY_SIGNATURE) {
        return Err(bad());
    }
    let (name_len, extra, comment) = (field(28, 2)?, field(30, 2)?, field(32, 2)?);
    let name = String::from_utf8_lossy(directory.get(at + 46..at + 46 + name_len).ok_or_else(bad)?).into_owned();
    let (csize, size, offset) = (field(20, 4)?, field(24, 4)?, field(42, 4)?);
    if [csize, size, offset].contains(&0xFFFF_FFFF) {
        return Err(ErrorKind::Unsupported.into());
    }
    let unix = directory.get(at + 5) == Some(&3);
    let mode = if unix { (field(38, 4)? >> 16) as u32 } else { 0 };
    let member = ZipMember {
        name,
        method: field(10, 2)? as u16,
        flags: field(8, 2)? as u16,
        crc: field(16, 4)? as u32,
        csize: csize as u64,
        size: size as u64,
        offset: offset as u64,
        mode,
    };
    Ok((member, at + 46 + name_len + extra + comment))
}

/// Toutes les entrées de l'archive (100 000 au plus).
pub fn zip_members(file: &mut File) -> io::Result<Vec<ZipMember>> {
    let (total, size, offset) = end_record(file)?;
    if total > MAX_MEMBERS || size > (64 << 20) {
        return Err(ErrorKind::InvalidData.into());
    }
    file.seek(SeekFrom::Start(offset))?;
    let mut directory = vec![0u8; size];
    file.read_exact(&mut directory)?;
    let (mut members, mut at) = (Vec::new(), 0);
    for _ in 0..total {
        let (member, next) = parse_member(&directory, at)?;
        members.push(member);
        at = next;
    }
    Ok(members)
}

/// Position des données d'une entrée : après son en-tête local, dont la longueur varie.
pub fn data_start(file: &mut File, member: &ZipMember) -> io::Result<u64> {
    file.seek(SeekFrom::Start(member.offset))?;
    let mut head = [0u8; 30];
    file.read_exact(&mut head)?;
    let lengths = (le(&head, 26, 2), le(&head, 28, 2));
    match lengths {
        (Some(name), Some(extra)) if head[..4] == LOCAL_SIGNATURE => Ok(member.offset + 30 + name as u64 + extra as u64),
        _ => Err(ErrorKind::InvalidData.into()),
    }
}
