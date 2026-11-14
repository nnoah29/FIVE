//! Remplissage des informations coûteuses d'un contexte : seulement celles que les règles demandent.

use std::fs;
use std::path::Path;
use std::time::Duration;

use super::context::FileContext;
use super::context_build::BuildEnv;
use super::info_dups::{Target, duplicate_info};
use super::info_files::{archive_of, git_status_of, hash_of, origin_of, text_of};
use super::info_image::image_info_of;

const DUPLICATE_BUDGET: Duration = Duration::from_secs(3);
const COUNT_LIMIT: usize = 20_000;

/// Nombre d'enfants directs, de fichiers (récursif) et taille cumulée d'un dossier, bornés pour rester rapides.
fn folder_counts(full: &Path) -> (u64, u64, u64) {
    let children = fs::read_dir(full).map_or(0, |d| d.count() as u64);
    let (mut files, mut size, mut seen) = (0u64, 0u64, 0usize);
    let mut pending = vec![full.to_path_buf()];
    while let Some(folder) = pending.pop() {
        for entry in fs::read_dir(&folder).into_iter().flatten().flatten() {
            seen += 1;
            let Ok(meta) = entry.metadata() else { continue };
            if meta.is_dir() && !meta.file_type().is_symlink() {
                pending.push(entry.path());
            } else {
                (files, size) = (files + 1, size + meta.len());
            }
            if seen > COUNT_LIMIT {
                return (children, files, size);
            }
        }
    }
    (children, files, size)
}

/// Indique si un dossier `.git` existe dans le dossier ou l'un de ses parents sous la racine.
fn inside_git(env: &BuildEnv, full: &Path) -> bool {
    full.ancestors().take_while(|a| env.root.inside(a)).any(|a| a.join(".git").exists())
}

/// Remplit ce que les règles utilisent : texte, image, hachage, archive, Git, origine, doublons, effectifs d'un dossier.
pub fn fill_costly(env: &BuildEnv, full: &Path, ctx: &mut FileContext) {
    let needs = env.needs;
    ctx.in_git = needs.git && inside_git(env, full);
    if ctx.is_dir {
        if needs.children {
            let (children, files, size) = folder_counts(full);
            (ctx.child_count, ctx.file_count, ctx.folder_size) = (Some(children), Some(files), Some(size));
        }
        return;
    }
    if needs.text {
        (ctx.text, ctx.encoding) = text_of(full);
    }
    ctx.image = if needs.image { image_info_of(full) } else { None };
    ctx.hash = if needs.hash { hash_of(full) } else { None };
    ctx.archive = (needs.archive && ctx.kind == "archive").then(|| archive_of(full));
    ctx.git_status = if needs.git { git_status_of(full) } else { None };
    ctx.origin_url = if needs.origin { origin_of(full) } else { None };
    if needs.duplicates {
        let target = Target { path: &ctx.path, size: ctx.size, mtime: ctx.modified.unwrap_or(0.0) };
        ctx.duplicates = duplicate_info(env.root, full, &target, env.watches, DUPLICATE_BUDGET);
    }
}
