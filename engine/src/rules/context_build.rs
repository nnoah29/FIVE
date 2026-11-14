//! Construction du contexte d'un fichier depuis le disque : métadonnées toujours, le reste selon les besoins des règles.

use std::collections::HashSet;
use std::fs::{self, Metadata};
use std::os::unix::fs::MetadataExt;
use std::time::UNIX_EPOCH;

use super::context::FileContext;
use super::context_fill::fill_costly;
use super::needs::Needs;
use super::types_rule::Watch;
use crate::disks::disk_space;
use crate::mime::guess;
use crate::mounts::read_mounts;
use crate::root::Root;

/// Ce que l'interface sait et le moteur non : favoris, récents, dossiers épinglés (chemins relatifs).
#[derive(Debug, Clone, Default)]
pub struct Extras {
    pub favorites: HashSet<String>,
    pub recents: HashSet<String>,
    pub pinned: HashSet<String>,
}

/// Tout ce qu'il faut pour construire des contextes pendant une passe.
pub struct BuildEnv<'a> {
    pub root: &'a Root,
    pub needs: Needs,
    pub extras: &'a Extras,
    pub watches: &'a [Watch],
    pub disks: &'a [String],
    pub now: f64,
}

/// Date d'une métadonnée en secondes Unix.
fn seconds(time: std::io::Result<std::time::SystemTime>) -> Option<f64> {
    time.ok().and_then(|t| t.duration_since(UNIX_EPOCH).ok()).map(|d| d.as_secs_f64())
}

/// Remplit les champs qui ne coûtent qu'un appel système : taille, dates, droits, nature, disque.
fn basic(env: &BuildEnv, rel: &str, full: &std::path::Path, meta: &Metadata) -> FileContext {
    let mut ctx = FileContext::new(rel);
    let target = fs::metadata(full).ok();
    ctx.now = env.now;
    ctx.is_symlink = meta.file_type().is_symlink();
    ctx.broken_symlink = ctx.is_symlink && target.is_none();
    ctx.is_dir = target.as_ref().is_some_and(Metadata::is_dir) && !ctx.is_symlink;
    ctx.size = meta.len();
    (ctx.modified, ctx.created, ctx.accessed) = (seconds(meta.modified()), seconds(meta.created()), seconds(meta.accessed()));
    ctx.mode = Some(meta.mode());
    ctx.executable = !ctx.is_dir && meta.mode() & 0o111 != 0;
    ctx.read_only = meta.mode() & 0o222 == 0;
    ctx.empty = if ctx.is_dir { fs::read_dir(full).is_ok_and(|mut d| d.next().is_none()) } else { meta.len() == 0 };
    ctx.mime = (!ctx.is_dir).then(|| guess(&ctx.name).to_string());
    ctx
}

/// Nom du disque qui contient le fichier : le point de montage le plus long qui le précède (« Système » pour `/`).
fn disk_of(full: &std::path::Path) -> Option<String> {
    let mount = read_mounts().into_iter().filter(|m| full.starts_with(&m.point)).max_by_key(|m| m.point.len())?;
    Some(if mount.point == "/" { "Système".to_string() } else { mount.point.rsplit('/').next().unwrap_or("").to_string() })
}

/// Construit le contexte du fichier `rel` ; `None` s'il a disparu ou sort de la racine.
pub fn build_context(env: &BuildEnv, rel: &str, stable_for: Option<f64>, counter: u32) -> Option<FileContext> {
    let full = env.root.resolve_entry(rel).ok()?;
    let meta = fs::symlink_metadata(&full).ok()?;
    let mut ctx = basic(env, rel, &full, &meta);
    ctx.stable_for = stable_for;
    ctx.counter = counter;
    ctx.in_trash = env.root.in_trash(&full);
    ctx.owned_by_me = fs::metadata(env.root.home()).is_ok_and(|home| home.uid() == meta.uid());
    ctx.mounted_disks = env.disks.to_vec();
    ctx.disk = disk_of(&full);
    (ctx.free_folder, ctx.free_disk) = (full.parent().map(|p| disk_space(p).0), full.parent().map(|p| disk_space(p).0));
    ctx.mount_point = ctx.is_dir && full.parent().and_then(|p| fs::metadata(p).ok()).is_some_and(|p| p.dev() != meta.dev());
    ctx.favorite = env.extras.favorites.contains(rel);
    ctx.recent = env.extras.recents.contains(rel);
    ctx.pinned = env.extras.pinned.contains(rel);
    fill_costly(env, &full, &mut ctx);
    Some(ctx)
}
