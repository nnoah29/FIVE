//! Jeton de session aléatoire et comparaison en temps constant.

use std::fs::File;
use std::io::Read;

const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

/// Jeton de 32 caractères (base64 sûr pour les URL) tiré de `/dev/urandom` : 192 bits.
pub fn new_token() -> std::io::Result<String> {
    let mut bytes = [0u8; 24];
    File::open("/dev/urandom")?.read_exact(&mut bytes)?;
    let mut text = String::with_capacity(32);
    for group in bytes.chunks(3) {
        let value = group.iter().fold(0u32, |acc, byte| (acc << 8) | *byte as u32);
        for shift in [18, 12, 6, 0] {
            text.push(ALPHABET[(value >> shift & 63) as usize] as char);
        }
    }
    Ok(text)
}

/// Compare deux textes sans s'arrêter au premier octet différent.
pub fn same_secret(a: &str, b: &str) -> bool {
    let (a, b) = (a.as_bytes(), b.as_bytes());
    let diff = a.iter().zip(b).fold(a.len() ^ b.len(), |acc, (x, y)| acc | (x ^ y) as usize);
    diff == 0
}
