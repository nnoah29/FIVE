//! En-têtes d'une archive tar : champs, somme de contrôle, lecture des blocs de données.

use std::io::{self, Read};

pub const BLOCK: usize = 512;

/// Entier octal écrit en ASCII dans un champ d'en-tête.
pub fn octal(field: &[u8]) -> u64 {
    let text: String = field.iter().take_while(|b| **b != 0).map(|b| *b as char).collect();
    u64::from_str_radix(text.trim(), 8).unwrap_or(0)
}

/// Texte d'un champ terminé par un octet nul.
pub fn text(field: &[u8]) -> String {
    let end = field.iter().position(|b| *b == 0).unwrap_or(field.len());
    String::from_utf8_lossy(&field[..end]).into_owned()
}

/// Somme de contrôle d'un en-tête, le champ lui-même étant compté comme des espaces.
fn checksum(header: &[u8; BLOCK]) -> u64 {
    header.iter().enumerate().map(|(at, b)| if (148..156).contains(&at) { 32 } else { *b as u64 }).sum()
}

/// Lit un bloc d'en-tête ; `None` à la fin du flux ou sur un bloc vide, erreur si la somme de contrôle est fausse.
pub fn next_header(reader: &mut impl Read, first: bool) -> io::Result<Option<[u8; BLOCK]>> {
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
pub fn take_data(reader: &mut impl Read, size: u64, keep: bool) -> io::Result<Vec<u8>> {
    let padded = size.div_ceil(BLOCK as u64) * BLOCK as u64;
    let mut data = Vec::new();
    let copied = if keep { reader.take(padded).read_to_end(&mut data)? as u64 } else { io::copy(&mut reader.take(padded), &mut io::sink())? };
    if copied < padded {
        return Err(io::ErrorKind::UnexpectedEof.into());
    }
    data.truncate(size as usize);
    Ok(data)
}

/// Nom complet d'une entrée : nom long annoncé avant l'en-tête, sinon préfixe ustar et nom court.
pub fn entry_name(header: &[u8; BLOCK], long_name: Option<String>) -> String {
    let prefix = text(&header[345..500]);
    let name = long_name.unwrap_or_else(|| text(&header[..100]));
    if !prefix.is_empty() && header[257..262] == *b"ustar" { format!("{prefix}/{name}") } else { name }
}
