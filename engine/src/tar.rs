//! Lecture des en-têtes d'une archive tar (éventuellement compressée), sans rien extraire.

use std::io::{self, Read};

use crate::zip::Member;

const BLOCK: usize = 512;

/// Entier octal écrit en ASCII dans un champ d'en-tête.
fn octal(field: &[u8]) -> u64 {
    let text: String = field.iter().take_while(|b| **b != 0).map(|b| *b as char).collect();
    u64::from_str_radix(text.trim(), 8).unwrap_or(0)
}

/// Texte d'un champ terminé par un octet nul.
fn text(field: &[u8]) -> String {
    let end = field.iter().position(|b| *b == 0).unwrap_or(field.len());
    String::from_utf8_lossy(&field[..end]).into_owned()
}

/// Somme de contrôle d'un en-tête, le champ lui-même étant compté comme des espaces.
fn checksum(header: &[u8; BLOCK]) -> u64 {
    header.iter().enumerate().map(|(at, b)| if (148..156).contains(&at) { 32 } else { *b as u64 }).sum()
}

/// Lit un bloc d'en-tête ; `None` à la fin du flux ou sur un bloc vide, erreur si la somme de contrôle est fausse.
fn next_header(reader: &mut impl Read, first: bool) -> io::Result<Option<[u8; BLOCK]>> {
    let mut header = [0u8; BLOCK];
    if reader.read_exact(&mut header).is_err() {
        return if first { Err(io::ErrorKind::InvalidData.into()) } else { Ok(None) };
    }
    if header.iter().all(|b| *b == 0) {
        return Ok(None);
    }
    if checksum(&header) != octal(&header[148..156]) {
        return Err(io::ErrorKind::InvalidData.into());
    }
    Ok(Some(header))
}

/// Lit ou saute les données d'une entrée (arrondies à 512 octets) ; retourne leur contenu si `keep`.
fn take_data(reader: &mut impl Read, size: u64, keep: bool) -> io::Result<Vec<u8>> {
    let padded = size.div_ceil(BLOCK as u64) * BLOCK as u64;
    let mut data = Vec::new();
    let copied = if keep { reader.take(padded).read_to_end(&mut data)? as u64 } else { io::copy(&mut reader.take(padded), &mut io::sink())? };
    if copied < padded {
        return Err(io::ErrorKind::UnexpectedEof.into());
    }
    data.truncate(size as usize);
    Ok(data)
}

/// Entrées d'un flux tar (au plus `limit`) et indicateur d'entrées en plus.
pub fn tar_entries(mut reader: impl Read, limit: usize) -> io::Result<(Vec<Member>, bool)> {
    let (mut out, mut long_name) = (Vec::new(), None::<String>);
    while let Some(header) = next_header(&mut reader, out.is_empty())? {
        let (size, kind) = (octal(&header[124..136]), header[156]);
        if matches!(kind, b'L' | b'x' | b'g') {
            let data = take_data(&mut reader, size, true)?;
            long_name = (kind == b'L').then(|| text(&data));
            continue;
        }
        if out.len() >= limit {
            return Ok((out, true));
        }
        let prefix = text(&header[345..500]);
        let mut name = long_name.take().unwrap_or_else(|| text(&header[..100]));
        if !prefix.is_empty() && header[257..262] == *b"ustar" {
            name = format!("{prefix}/{name}");
        }
        out.push((name, size, kind == b'5'));
        take_data(&mut reader, size, false)?;
    }
    Ok((out, false))
}
