//! Reconnaissance du format d'une archive qu'on sait extraire : zip, tar, tar.gz.

use std::fs::File;
use std::io::Read;
use std::path::Path;

use flate2::read::GzDecoder;

use crate::tar_header::{BLOCK, next_header};
use crate::zip::is_zip;

/// Format d'archive extractible.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArchiveKind {
    Zip,
    Tar,
    TarGz,
}

/// Indique si le bloc commence par un en-tête tar valide.
fn is_tar_head(block: &[u8]) -> bool {
    let mut reader = block;
    block.len() == BLOCK && matches!(next_header(&mut reader, true), Ok(Some(_)))
}

/// Lit jusqu'à un bloc de 512 octets au début de `reader`.
fn first_block(mut reader: impl Read) -> Vec<u8> {
    let mut block = vec![0u8; BLOCK];
    let mut filled = 0;
    while filled < BLOCK {
        match reader.read(&mut block[filled..]) {
            Ok(0) | Err(_) => break,
            Ok(read) => filled += read,
        }
    }
    block.truncate(filled);
    block
}

/// Reconnaît le format d'après le contenu du fichier ; `None` si ce n'est pas une archive prise en charge.
pub fn detect(path: &Path) -> Option<ArchiveKind> {
    let head = first_block(File::open(path).ok()?);
    if is_zip(&head) {
        return Some(ArchiveKind::Zip);
    }
    if head.starts_with(&[0x1f, 0x8b]) {
        return is_tar_head(&first_block(GzDecoder::new(File::open(path).ok()?))).then_some(ArchiveKind::TarGz);
    }
    is_tar_head(&head).then_some(ArchiveKind::Tar)
}
