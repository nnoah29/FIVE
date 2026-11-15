//! Chemins du portail : octets terminés par un zéro, dans les deux sens.

use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;

use gtk4::glib::Variant;
use gtk4::prelude::*;

/// Convertit un chemin du portail (octets terminés par un zéro) en chemin.
pub fn path_from_bytes(bytes: &[u8]) -> PathBuf {
    let end = bytes.iter().position(|b| *b == 0).unwrap_or(bytes.len());
    PathBuf::from(OsStr::from_bytes(&bytes[..end]))
}

/// Convertit un chemin en octets terminés par un zéro, comme le portail les attend.
pub fn bytes_of(path: &std::path::Path) -> Variant {
    let mut bytes = path.as_os_str().as_bytes().to_vec();
    bytes.push(0);
    bytes.to_variant()
}
