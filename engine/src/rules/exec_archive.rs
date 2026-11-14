//! Exécution des archives : compresser en zip (avec nom choisi), extraire, mettre l'original à la corbeille.

use std::fs;
use std::path::Path;

use super::exec_common::{run_job, target_free};
use super::outcome::PlannedAction;
use super::run_types::{Done, Fail, RunEnv, Step, Undo};
use super::types_action::AfterUnzip;
use super::types_rule::ConflictPolicy;
use crate::jobs::compress::zip_name;
use crate::jobs::extract::{folder_name, unique_folder};
use crate::trash_ops::move_to_trash_at;

/// Dossier parent (relatif à la racine) d'un chemin relatif.
fn parent_rel(path: &str) -> String {
    path.rsplit_once('/').map_or_else(String::new, |(parent, _)| parent.to_string())
}

/// Crée une archive zip de `paths` à côté du premier ; un nom choisi remplace le nom par défaut. Retourne le chemin de l'archive.
pub fn make_zip(env: &RunEnv, paths: &[String], name: Option<&str>, policy: ConflictPolicy) -> Result<std::path::PathBuf, Fail> {
    let first = env.root.resolve_entry(paths.first().ok_or_else(|| Fail::Error("Rien à compresser".into()))?)?;
    let folder = first.parent().map(Path::to_path_buf).unwrap_or_default();
    let sources: Vec<_> = paths.iter().filter_map(|p| env.root.resolve_entry(p).ok()).collect();
    let made = crate::jobs::copy::unique_destination(&folder, &zip_name(&sources));
    run_job(env, "zip", paths, &env.root.relative(&folder))?;
    let Some(wanted) = name else { return Ok(made) };
    let wanted = if wanted.to_lowercase().ends_with(".zip") { wanted.to_string() } else { format!("{wanted}.zip") };
    let target = target_free(&folder, &crate::names::clean_name(&wanted).map_err(Fail::from)?, policy)?;
    fs::rename(&made, &target)?;
    Ok(target)
}

/// Compresse un élément ; `then_trash` met ensuite l'original à la corbeille.
pub fn zip_one(env: &RunEnv, planned: &PlannedAction, current: &str, then_trash: bool) -> Step {
    let archive = make_zip(env, &[current.to_string()], planned.text.as_deref(), planned.on_conflict)?;
    let mut undo = vec![Undo::TrashCreated { path: archive.to_string_lossy().into_owned() }];
    if then_trash {
        let full = env.root.resolve_entry(current)?;
        let trashed = move_to_trash_at(env.root, &full)?;
        undo.push(Undo::Trashed { original: full.to_string_lossy().into_owned(), trashed: trashed.to_string_lossy().into_owned() });
    }
    Ok(Done { detail: env.root.relative(&archive), undo })
}

/// Extrait une archive dans un dossier neuf à côté d'elle ; l'archive peut ensuite aller à la corbeille.
pub fn unzip_one(env: &RunEnv, current: &str, then: AfterUnzip) -> Step {
    let archive = env.root.resolve_entry(current)?;
    let folder = unique_folder(archive.parent().unwrap_or(Path::new("/")), &folder_name(&archive));
    run_job(env, "unzip", &[current.to_string()], &parent_rel(current))?;
    let mut undo = vec![Undo::TrashCreated { path: folder.to_string_lossy().into_owned() }];
    if then == AfterUnzip::TrashArchive {
        let trashed = move_to_trash_at(env.root, &archive)?;
        undo.push(Undo::Trashed { original: archive.to_string_lossy().into_owned(), trashed: trashed.to_string_lossy().into_owned() });
    }
    Ok(Done { detail: env.root.relative(&folder), undo })
}
