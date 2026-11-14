//! Exécution d'un transfert : copie, ou déplacement par renommage avec repli sur copie.

use std::fs;
use std::path::Path;
use std::sync::Arc;

use super::compress::run_zip;
use super::copy::{copy_tree, unique_destination};
use super::extract::run_unzip;
use super::job::Job;
use super::measure::measure;
use super::state::{Kind, State, Stop};

/// Supprime la source d'un déplacement qui est passé par une copie.
fn remove_source(path: &Path) -> std::io::Result<()> {
    if fs::symlink_metadata(path)?.is_dir() { fs::remove_dir_all(path) } else { fs::remove_file(path) }
}

/// Transfère un élément vers le dossier de destination du transfert.
fn transfer_one(job: &Job, src: &Path) -> Result<(), Stop> {
    let name = src.file_name().map_or_else(String::new, |n| n.to_string_lossy().into_owned());
    let dst = unique_destination(&job.dest, &name);
    if job.kind == Kind::Move && fs::rename(src, &dst).is_ok() {
        let (size, count) = measure(&dst);
        job.update(|p| {
            p.done += size;
            p.files_done += count;
        });
        return Ok(());
    }
    copy_tree(job, src, &dst)?;
    if job.kind == Kind::Move {
        remove_source(src)?;
    }
    Ok(())
}

/// Exécute le travail du transfert selon sa nature : copie ou déplacement par source, une seule archive pour `zip`, une extraction par archive pour `unzip`.
fn run_steps(job: &Job) -> Result<(), Stop> {
    match job.kind {
        Kind::Zip => run_zip(job),
        Kind::Unzip => run_unzip(job),
        Kind::Copy | Kind::Move => job.sources.iter().try_for_each(|src| {
            job.checkpoint()?;
            transfer_one(job, src)
        }),
    }
}

/// Corps du thread de transfert : exécute le travail puis fixe l'état final.
pub fn run_job(job: Arc<Job>) {
    let outcome = run_steps(&job);
    job.update(|p| {
        p.current.clear();
        match outcome {
            Ok(()) => (p.state, p.done) = (State::Done, p.total),
            Err(Stop::Cancelled) => p.state = State::Cancelled,
            Err(Stop::Io(error)) => {
                p.state = State::Error;
                p.error = crate::refus::Refus::from(error).message;
            }
        }
    });
}
