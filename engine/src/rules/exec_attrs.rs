//! Exécution des actions sur les attributs : droits, date de modification, création de dossier.

use std::fs::{self, File, Permissions};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::time::{Duration, UNIX_EPOCH};

use super::exec_common::safe_relative;
use super::outcome::PlannedAction;
use super::run_types::{Done, Fail, RunEnv, Step, Undo};
use super::time_calc::parse_local;
use super::types_action::RuleAction as A;

/// Change les droits avec `edit` ; refuse les liens symboliques ; retourne l'annulation (anciens droits).
fn change_mode(env: &RunEnv, current: &str, edit: impl Fn(u32) -> u32) -> Step {
    let full = env.root.resolve_entry(current)?;
    let meta = fs::symlink_metadata(&full)?;
    if meta.file_type().is_symlink() {
        return Err(Fail::Skip("lien symbolique ignoré".into()));
    }
    let (old, new) = (meta.permissions().mode() & 0o7777, edit(meta.permissions().mode()) & 0o777);
    fs::set_permissions(&full, Permissions::from_mode(new))?;
    Ok(Done { detail: format!("{new:o}"), undo: vec![Undo::Mode { path: full.to_string_lossy().into_owned(), mode: old }] })
}

/// Fixe la date de modification à la date calculée par la règle.
fn set_time(env: &RunEnv, current: &str, text: Option<&str>) -> Step {
    let seconds = text.and_then(parse_local).ok_or_else(|| Fail::Error("Date introuvable pour ce fichier".into()))?;
    let full = env.root.resolve_entry(current)?;
    let old = fs::metadata(&full)?.modified()?.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs_f64();
    File::open(&full)?.set_modified(UNIX_EPOCH + Duration::from_secs_f64(seconds.max(0.0)))?;
    Ok(Done { detail: text.unwrap_or("").to_string(), undo: vec![Undo::Time { path: full.to_string_lossy().into_owned(), seconds: old }] })
}

/// Crée un dossier (ou une suite de dossiers) à côté du fichier ; refuse de sortir de la racine.
fn create_folder(env: &RunEnv, current: &str, text: Option<&str>) -> Step {
    let name = safe_relative(text.unwrap_or(""))?;
    let base = env.root.resolve_entry(current)?;
    let target = base.parent().map(|p| p.join(&name)).unwrap_or_default();
    if name.is_empty() || !env.root.inside(&target) || Path::new(&target).exists() {
        return Err(Fail::Skip("dossier déjà présent ou nom vide".into()));
    }
    fs::create_dir_all(&target)?;
    Ok(Done { detail: env.root.relative(&target), undo: vec![Undo::RemoveEmptyDir { path: target.to_string_lossy().into_owned() }] })
}

/// Exécute une action d'attributs ; `None` pour les autres actions.
pub fn run(env: &RunEnv, planned: &PlannedAction, current: &str) -> Option<Step> {
    Some(match &planned.action {
        A::SetReadOnly { value } => change_mode(env, current, |m| if *value { m & !0o222 } else { m | 0o200 }),
        A::SetExecutable { value } => change_mode(env, current, |m| if *value { m | ((m & 0o444) >> 2) } else { m & !0o111 }),
        A::SetPermissions { mode } if *mode <= 0o777 => change_mode(env, current, |_| *mode),
        A::SetPermissions { .. } => Err(Fail::Error("Droits spéciaux (setuid, setgid) refusés".into())),
        A::SetModifiedTime { .. } => set_time(env, current, planned.text.as_deref()),
        A::CreateFolder { .. } => create_folder(env, current, planned.text.as_deref()),
        _ => return None,
    })
}
