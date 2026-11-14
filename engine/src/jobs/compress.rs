//! Création d'une archive zip depuis les éléments d'un transfert.

use std::fs;
use std::path::{Path, PathBuf};

use super::copy::unique_destination;
use super::job::Job;
use super::state::Stop;
use crate::zip_write::ZipOut;

/// Nom d'un chemin, sans ses dossiers.
fn base_name(path: &Path) -> String {
    path.file_name().map_or_else(String::new, |n| n.to_string_lossy().into_owned())
}

/// Nom de l'archive : celui de l'élément seul suivi de `.zip`, sinon `Archive.zip`.
pub fn zip_name(sources: &[PathBuf]) -> String {
    match sources {
        [only] => format!("{}.zip", base_name(only)),
        _ => "Archive.zip".to_string(),
    }
}

/// Ajoute un élément et son contenu sous `name` ; liens, fichiers spéciaux et l'archive elle-même sont ignorés.
fn add_tree(job: &Job, out: &mut ZipOut, src: &Path, name: &str, skip: &Path) -> Result<(), Stop> {
    job.checkpoint()?;
    let meta = fs::symlink_metadata(src)?;
    if src == skip || !(meta.is_file() || meta.is_dir()) {
        return Ok(());
    }
    if meta.is_file() {
        out.add_file(job, name, src)?;
        job.update(|p| p.files_done += 1);
        return Ok(());
    }
    out.add_dir(&format!("{name}/"), &meta)?;
    let mut children: Vec<_> = fs::read_dir(src)?.flatten().collect();
    children.sort_by_key(|child| child.file_name());
    children.iter().try_for_each(|child| add_tree(job, out, &child.path(), &format!("{name}/{}", child.file_name().to_string_lossy()), skip))
}

/// Écrit l'archive complète : chaque source, puis le répertoire central.
fn write_archive(job: &Job, target: &Path) -> Result<(), Stop> {
    let mut out = ZipOut::create(target)?;
    for src in &job.sources {
        add_tree(job, &mut out, src, &base_name(src), target)?;
    }
    Ok(out.finish()?)
}

/// Crée l'archive du transfert dans son dossier de destination ; un échec ou une annulation supprime l'archive partielle.
pub fn run_zip(job: &Job) -> Result<(), Stop> {
    let target = unique_destination(&job.dest, &zip_name(&job.sources));
    let result = write_archive(job, &target);
    if result.is_err() {
        let _ = fs::remove_file(&target);
    }
    result
}
