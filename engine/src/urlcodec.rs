//! Encodage et décodage pourcent des chemins (fichiers `.trashinfo`, URI `file://`).

/// Encode un chemin : garde les lettres, chiffres, `/` et `_.-~`, encode le reste en UTF-8.
pub fn encode_path(path: &str) -> String {
    let mut out = String::with_capacity(path.len());
    for byte in path.bytes() {
        if byte.is_ascii_alphanumeric() || b"/_.-~".contains(&byte) {
            out.push(byte as char);
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}

/// Décode les séquences `%XX` ; les séquences invalides sont laissées telles quelles.
pub fn decode(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut at = 0;
    while at < bytes.len() {
        match hex_pair(bytes, at) {
            Some(value) => {
                out.push(value);
                at += 3;
            }
            None => {
                out.push(bytes[at]);
                at += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Lit `%XX` à la position donnée, si présent.
fn hex_pair(bytes: &[u8], at: usize) -> Option<u8> {
    if bytes.get(at) != Some(&b'%') {
        return None;
    }
    let pair = std::str::from_utf8(bytes.get(at + 1..at + 3)?).ok()?;
    u8::from_str_radix(pair, 16).ok()
}
