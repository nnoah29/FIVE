//! Conditions sur la nature, la famille de fichier et la taille.

use super::context::FileContext;
use super::matcher::{compare, text_matches};
use super::types_condition::ConditionTest as T;

/// Évalue une condition de nature (fichier, dossier, lien, droits) ; `None` hors de cette famille.
pub fn eval_nature(ctx: &FileContext, test: &T) -> Option<bool> {
    Some(match test {
        T::IsFile => !ctx.is_dir,
        T::IsFolder => ctx.is_dir,
        T::IsSymlink => ctx.is_symlink,
        T::IsBrokenSymlink => ctx.broken_symlink,
        T::IsHidden => ctx.hidden(),
        T::IsExecutable => ctx.executable,
        T::IsReadOnly => ctx.read_only,
        T::IsEmpty => ctx.empty,
        T::IsMountPoint => ctx.mount_point,
        _ => return None,
    })
}

/// Évalue une condition de famille ou de type MIME ; `None` hors de cette famille.
pub fn eval_kind(ctx: &FileContext, test: &T) -> Option<bool> {
    Some(match test {
        T::Kind { in_list } => in_list.iter().any(|kind| kind.eq_ignore_ascii_case(&ctx.kind)),
        T::Mime { matcher } => ctx.mime.as_deref().is_some_and(|mime| text_matches(matcher, mime)),
        _ => return None,
    })
}

/// Évalue une condition de taille ou de contenu d'un dossier ; une information absente rend la condition fausse.
pub fn eval_size(ctx: &FileContext, test: &T) -> Option<bool> {
    let check = |cmp, value: Option<u64>, reference: u64| value.is_some_and(|v| compare(cmp, v as f64, reference as f64));
    Some(match test {
        T::Size { cmp, bytes } => !ctx.is_dir && check(*cmp, Some(ctx.size), *bytes),
        T::FolderTotalSize { cmp, bytes } => check(*cmp, ctx.folder_size, *bytes),
        T::ChildCount { cmp, n } => check(*cmp, ctx.child_count, *n),
        T::FileCount { cmp, n } => check(*cmp, ctx.file_count, *n),
        _ => return None,
    })
}
