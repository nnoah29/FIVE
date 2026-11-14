//! En-têtes d'une archive zip à écrire : entrée locale, répertoire central, fin d'archive.

use crate::clock::local_fields;

/// Entrée d'archive : champs connus après l'écriture de ses données.
#[derive(Debug, Clone)]
pub struct ZipEntry {
    pub name: String,
    pub method: u16,
    pub crc: u32,
    pub csize: u32,
    pub size: u32,
    pub offset: u32,
    pub stamp: (u16, u16),
    pub mode: u32,
}

/// Ajoute `value` sur `width` octets, poids faible d'abord.
fn put(out: &mut Vec<u8>, value: u64, width: usize) {
    out.extend(value.to_le_bytes().iter().take(width));
}

/// Heure et date DOS d'un instant en secondes Unix, à l'heure locale.
pub fn dos_stamp(seconds: f64) -> (u16, u16) {
    let (year, month, day, hour, minute) = local_fields(seconds);
    let date = (year.clamp(1980, 2107) as u16 - 1980) << 9 | (month as u16) << 5 | day as u16;
    ((hour as u16) << 11 | (minute as u16) << 5, date)
}

/// Crc, taille compressée et taille d'origine : les 12 octets reportés dans l'en-tête local après l'écriture.
pub fn sizes_field(entry: &ZipEntry) -> Vec<u8> {
    let mut out = Vec::new();
    [entry.crc, entry.csize, entry.size].iter().for_each(|v| put(&mut out, *v as u64, 4));
    out
}

/// En-tête local (noms en UTF-8) ; un fichier l'écrit avec des tailles à 0, corrigées ensuite.
pub fn local_header(entry: &ZipEntry) -> Vec<u8> {
    let mut out = vec![0x50, 0x4b, 0x03, 0x04];
    [(20, 2), (0x0800, 2), (entry.method as u64, 2), (entry.stamp.0 as u64, 2), (entry.stamp.1 as u64, 2)].iter().for_each(|(v, w)| put(&mut out, *v, *w));
    out.extend(sizes_field(entry));
    put(&mut out, entry.name.len() as u64, 2);
    put(&mut out, 0, 2);
    out.extend(entry.name.as_bytes());
    out
}

/// Entrée du répertoire central, avec les droits Unix dans les attributs externes.
pub fn central_header(entry: &ZipEntry) -> Vec<u8> {
    let mut out = vec![0x50, 0x4b, 0x01, 0x02];
    [(0x031e, 2), (20, 2), (0x0800, 2), (entry.method as u64, 2), (entry.stamp.0 as u64, 2), (entry.stamp.1 as u64, 2)]
        .iter()
        .for_each(|(v, w)| put(&mut out, *v, *w));
    out.extend(sizes_field(entry));
    [(entry.name.len() as u64, 2), (0, 2), (0, 2), (0, 2), (0, 2), ((entry.mode as u64) << 16, 4), (entry.offset as u64, 4)]
        .iter()
        .for_each(|(v, w)| put(&mut out, *v, *w));
    out.extend(entry.name.as_bytes());
    out
}

/// Fin d'archive : nombre d'entrées, taille et position du répertoire central.
pub fn end_record(count: u16, size: u32, offset: u32) -> Vec<u8> {
    let mut out = vec![0x50, 0x4b, 0x05, 0x06];
    [(0, 2), (0, 2), (count as u64, 2), (count as u64, 2), (size as u64, 4), (offset as u64, 4), (0, 2)].iter().for_each(|(v, w)| put(&mut out, *v, *w));
    out
}
