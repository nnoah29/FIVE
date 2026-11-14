//! Extraction des archives d'un transfert : un dossier neuf par archive, supprimé si l'extraction échoue.

use std::fs::{self, File};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use flate2::read::GzDecoder;

use super::counted::Counted;
use super::job::Job;
use super::state::Stop;
use super::untar_run::extract_tar;
use super::unzip_run::extract_zip;
use crate::archive_kind::{ArchiveKind, detect};

/// Nom du dossier d'extraction : celui de l'archive sans son extension (.zip, .tar, .tar.gz, .tgz).
pub fn folder_name(archive: &Path) -> String {
    let name = archive.file_name().map_or_else(String::new, |n| n.to_string_lossy().into_owned());
    let lower = name.to_lowercase();
    let cut = [".tar.gz", ".tgz", ".tar", ".zip"].iter().find(|ext| lower.ends_with(*ext)).map_or(0, |ext| ext.len());
    let stem = name[..name.len() - cut].to_string();
    if stem.is_empty() { "Archive".to_string() } else { stem }
}

/// Premier dossier libre sous `dest` : `nom`, puis `nom (2)`, `nom (3)`…
pub fn unique_folder(dest: &Path, name: &str) -> PathBuf {
    let mut candidate = dest.join(name);
    let mut n = 1;
    while fs::symlink_metadata(&candidate).is_ok() {
        n += 1;
        candidate = dest.join(format!("{name} ({n})"));
    }
    candidate
}

/// Extrait selon le format ; la progression compte les octets lus de l'archive.
fn extract_kind(job: &Job, kind: ArchiveKind, archive: &Path, folder: &Path) -> Result<(), Stop> {
    match kind {
        ArchiveKind::Zip => extract_zip(job, archive, folder),
        ArchiveKind::Tar => extract_tar(job, Counted { inner: File::open(archive)?, job }, folder),
        ArchiveKind::TarGz => extract_tar(job, GzDecoder::new(Counted { inner: File::open(archive)?, job }), folder),
    }
}

/// Extrait une archive dans un dossier neuf à côté de la destination du transfert.
fn extract_one(job: &Job, archive: &Path) -> Result<(), Stop> {
    let kind = detect(archive).ok_or_else(|| Stop::Io(ErrorKind::Unsupported.into()))?;
    let folder = unique_folder(&job.dest, &folder_name(archive));
    fs::create_dir(&folder)?;
    job.update(|p| p.current = archive.file_name().map_or_else(String::new, |n| n.to_string_lossy().into_owned()));
    let result = extract_kind(job, kind, archive, &folder);
    if result.is_err() {
        let _ = fs::remove_dir_all(&folder);
    }
    result
}

/// Extrait chaque archive du transfert l'une après l'autre.
pub fn run_unzip(job: &Job) -> Result<(), Stop> {
    for archive in &job.sources {
        job.checkpoint()?;
        extract_one(job, archive)?;
        job.update(|p| p.files_done += 1);
    }
    Ok(())
}
