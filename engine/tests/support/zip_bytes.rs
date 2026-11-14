//! Fabrique d'archives zip à une entrée, avec somme de contrôle et taille annoncée au choix, pour les tests de sécurité.

fn put(out: &mut Vec<u8>, value: u32, width: usize) {
    out.extend(value.to_le_bytes().iter().take(width));
}

pub fn stored_zip(name: &str, data: &[u8], crc: u32, declared: u32) -> Vec<u8> {
    let (mut body, mut directory) = (vec![0x50, 0x4b, 0x03, 0x04], vec![0x50, 0x4b, 0x01, 0x02]);
    for (value, width) in [(20, 2), (0, 2), (0, 2), (0, 2), (0, 2), (crc, 4), (data.len() as u32, 4), (declared, 4), (name.len() as u32, 2), (0, 2)] {
        put(&mut body, value, width);
    }
    body.extend(name.bytes().chain(data.iter().copied()));
    for (value, width) in [(20, 2), (20, 2), (0, 2), (0, 2), (0, 2), (0, 2), (crc, 4), (data.len() as u32, 4), (declared, 4), (name.len() as u32, 2)] {
        put(&mut directory, value, width);
    }
    for (value, width) in [(0, 2), (0, 2), (0, 2), (0, 2), (0, 4), (0, 4)] {
        put(&mut directory, value, width);
    }
    directory.extend(name.bytes());
    let start = body.len() as u32;
    body.extend(&directory);
    body.extend([0x50, 0x4b, 0x05, 0x06, 0, 0, 0, 0, 1, 0, 1, 0]);
    put(&mut body, directory.len() as u32, 4);
    put(&mut body, start, 4);
    body.extend([0, 0]);
    body
}
