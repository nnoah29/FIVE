//! Contenu d'une archive zip, tar ou tar.gz, listé sans extraction.

use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::Path;

use flate2::read::GzDecoder;
use serde_json::{Value, json};

use crate::refus::{Refus, Resultat};
use crate::tar::tar_entries;
use crate::zip::{Member, is_zip, zip_entries};

const LIMIT: usize = 500;

/// Lit les premiers octets du fichier pour reconnaître son format.
fn sniff(file: &mut File) -> io::Result<Vec<u8>> {
    let mut head = vec![0u8; 4];
    let read = file.read(&mut head)?;
    head.truncate(read);
    file.seek(SeekFrom::Start(0))?;
    Ok(head)
}

/// Extrait les entrées selon le format : zip, tar compressé en gzip, ou tar.
fn members(file: &mut File) -> io::Result<(Vec<Member>, bool)> {
    let head = sniff(file)?;
    if is_zip(&head) {
        return zip_entries(file, LIMIT);
    }
    if head.starts_with(&[0x1f, 0x8b]) {
        return tar_entries(GzDecoder::new(&*file), LIMIT);
    }
    tar_entries(&*file, LIMIT)
}

/// Contenu d'une archive (500 entrées au plus) ; refuse un format illisible.
pub fn list_archive(full: &Path) -> Resultat<Value> {
    let mut file = File::open(full)?;
    let (rows, more) = members(&mut file).map_err(|_| Refus::new(400, "Archive illisible : format non reconnu ou fichier abîmé"))?;
    let items: Vec<Value> = rows.into_iter().map(|(n, s, d)| json!({"n": n, "s": s, "d": d})).collect();
    if items.is_empty() && !full.is_file() {
        return Err(Refus::new(400, "Ce n'est pas un fichier"));
    }
    Ok(json!({"items": items, "more": more}))
}
