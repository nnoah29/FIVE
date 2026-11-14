//! Dimensions d'une image lues dans son en-tête (PNG, GIF, BMP, JPEG, WebP), sans la décoder.

use std::fs::File;
use std::io::Read;
use std::path::Path;

use super::context::ImageInfo;

/// Lit un entier de `width` octets à `at`, gros-boutiste ou petit-boutiste.
fn int(bytes: &[u8], at: usize, width: usize, big: bool) -> Option<u64> {
    let slice = bytes.get(at..at + width)?;
    Some(if big { slice.iter().fold(0, |acc, b| (acc << 8) | *b as u64) } else { slice.iter().rev().fold(0, |acc, b| (acc << 8) | *b as u64) })
}

/// Cherche la marque de taille d'une image JPEG (segments SOF0 à SOF15).
fn jpeg(bytes: &[u8]) -> Option<ImageInfo> {
    let mut at = 2;
    while at + 9 < bytes.len() {
        let marker = *bytes.get(at + 1)?;
        if bytes[at] == 0xff && (0xc0..=0xcf).contains(&marker) && ![0xc4, 0xc8, 0xcc].contains(&marker) {
            return Some(ImageInfo { height: int(bytes, at + 5, 2, true)?, width: int(bytes, at + 7, 2, true)?, has_alpha: false });
        }
        at += 2 + int(bytes, at + 2, 2, true)? as usize;
    }
    None
}

/// Dimensions d'après les premiers octets : PNG, GIF, BMP, WebP (VP8X, VP8L simple) ou JPEG.
pub fn image_info(bytes: &[u8]) -> Option<ImageInfo> {
    if bytes.starts_with(b"\x89PNG") {
        return Some(ImageInfo { width: int(bytes, 16, 4, true)?, height: int(bytes, 20, 4, true)?, has_alpha: matches!(bytes.get(25), Some(4 | 6)) });
    }
    if bytes.starts_with(b"GIF8") {
        return Some(ImageInfo { width: int(bytes, 6, 2, false)?, height: int(bytes, 8, 2, false)?, has_alpha: true });
    }
    if bytes.starts_with(b"BM") {
        return Some(ImageInfo { width: int(bytes, 18, 4, false)?, height: int(bytes, 22, 4, false)?.max(1), has_alpha: false });
    }
    if bytes.starts_with(b"RIFF") && bytes.get(8..12) == Some(b"WEBP") && bytes.get(12..16) == Some(b"VP8X") {
        return Some(ImageInfo {
            width: int(bytes, 24, 3, false)? + 1,
            height: int(bytes, 27, 3, false)? + 1,
            has_alpha: bytes.get(20).is_some_and(|f| f & 0x10 != 0),
        });
    }
    bytes.starts_with(&[0xff, 0xd8]).then(|| jpeg(bytes)).flatten()
}

/// Lit l'en-tête d'un fichier image (256 Ko au plus) ; `None` si le format n'est pas reconnu.
pub fn image_info_of(path: &Path) -> Option<ImageInfo> {
    let mut head = Vec::new();
    File::open(path).ok()?.take(256 * 1024).read_to_end(&mut head).ok()?;
    image_info(&head)
}
