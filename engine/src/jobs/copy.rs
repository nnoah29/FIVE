//! Copie de fichiers et de dossiers, bloc par bloc, avec points d'arrêt.

use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};

use super::job::Job;
use super::state::Stop;

const CHUNK: usize = 1 << 20;

/// Premier nom libre dans `folder` : `nom`, puis `nom (2).ext`, `nom (3).ext`…
pub fn unique_destination(folder: &Path, name: &str) -> PathBuf {
    let (stem, ext) = match name.rsplit_once('.') {
        Some((stem, ext)) if !stem.is_empty() => (stem.to_string(), format!(".{ext}")),
        _ => (name.to_string(), String::new()),
    };
    let mut candidate = name.to_string();
    let mut n = 1;
    while fs::symlink_metadata(folder.join(&candidate)).is_ok() {
        n += 1;
        candidate = format!("{stem} ({n}){ext}");
    }
    folder.join(candidate)
}

/// Copie un fichier par blocs ; une annulation supprime le fichier partiel.
fn copy_blocks(job: &Job, reader: &mut File, writer: &mut File) -> Result<(), Stop> {
    let mut block = vec![0u8; CHUNK];
    loop {
        job.checkpoint()?;
        let read = reader.read(&mut block)?;
        if read == 0 {
            return Ok(());
        }
        writer.write_all(&block[..read])?;
        job.update(|p| p.done += read as u64);
    }
}

/// Copie un fichier ou un lien symbolique vers `dst`, qui ne doit pas exister.
pub fn copy_file(job: &Job, src: &Path, dst: &Path) -> Result<(), Stop> {
    let name = src.file_name().map_or_else(String::new, |n| n.to_string_lossy().into_owned());
    job.update(|p| p.current = name);
    if fs::symlink_metadata(src)?.file_type().is_symlink() {
        symlink(fs::read_link(src)?, dst)?;
    } else {
        let mut reader = File::open(src)?;
        let mut writer = OpenOptions::new().write(true).create_new(true).open(dst)?;
        if let Err(stop) = copy_blocks(job, &mut reader, &mut writer) {
            let _ = fs::remove_file(dst);
            return Err(stop);
        }
        keep_metadata(src, dst);
    }
    job.update(|p| p.files_done += 1);
    Ok(())
}

/// Reporte permissions et date de modification de la source sur la copie.
fn keep_metadata(src: &Path, dst: &Path) {
    let Ok(info) = fs::metadata(src) else { return };
    let _ = fs::set_permissions(dst, info.permissions());
    if let (Ok(file), Ok(modified)) = (File::open(dst), info.modified()) {
        let _ = file.set_modified(modified);
    }
}

/// Copie récursivement un élément ; les liens symboliques sont copiés comme liens.
pub fn copy_tree(job: &Job, src: &Path, dst: &Path) -> Result<(), Stop> {
    let info = fs::symlink_metadata(src)?;
    if info.file_type().is_symlink() || !info.is_dir() {
        return copy_file(job, src, dst);
    }
    fs::create_dir(dst)?;
    let mut names: Vec<_> = fs::read_dir(src)?.flatten().map(|e| e.file_name()).collect();
    names.sort();
    for name in names {
        job.checkpoint()?;
        copy_tree(job, &src.join(&name), &dst.join(&name))?;
    }
    keep_metadata(src, dst);
    Ok(())
}
