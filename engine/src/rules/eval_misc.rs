//! Conditions sur les doublons, Git, l'origine, le système et l'interface.

use super::context::{DuplicateGroup, FileContext};
use super::matcher::text_matches;
use super::types_condition::ConditionTest as T;

/// Groupe de doublons de la portée demandée (`folder`, `watched`, `root`).
fn group<'a>(ctx: &'a FileContext, scope: &str) -> Option<&'a DuplicateGroup> {
    let all = ctx.duplicates.as_ref()?;
    match scope {
        "folder" => Some(&all.folder),
        "watched" => Some(&all.watched),
        "root" => Some(&all.root),
        _ => None,
    }
}

/// Évalue une condition de doublon ; une portée inconnue ou des doublons non calculés rendent la condition fausse.
pub fn eval_duplicates(ctx: &FileContext, test: &T) -> Option<bool> {
    Some(match test {
        T::DuplicateOfAnother { scope } => group(ctx, scope).is_some_and(|g| g.others > 0),
        T::IsNewestOfDuplicates { scope } => group(ctx, scope).is_some_and(|g| g.others > 0 && g.newest),
        T::IsOldestOfDuplicates { scope } => group(ctx, scope).is_some_and(|g| g.others > 0 && g.oldest),
        T::SameNameAs { scope } => group(ctx, scope).is_some_and(|g| g.same_name > 0),
        _ => return None,
    })
}

/// Évalue une condition d'état Git, d'origine de téléchargement ou de disque d'origine.
pub fn eval_origin(ctx: &FileContext, test: &T) -> Option<bool> {
    Some(match test {
        T::GitStatus { in_list } => ctx.git_status.as_deref().is_some_and(|s| in_list.iter().any(|x| x.eq_ignore_ascii_case(s))),
        T::DownloadedFrom { matcher } => ctx.origin_url.as_deref().is_some_and(|url| text_matches(matcher, url)),
        T::FromDisk { disk } => ctx.disk.as_deref().is_some_and(|d| d.eq_ignore_ascii_case(disk)),
        _ => return None,
    })
}

/// Évalue une condition système : espace libre, disque monté, propriétaire, droits.
pub fn eval_system(ctx: &FileContext, test: &T) -> Option<bool> {
    Some(match test {
        T::FreeSpaceBelow { bytes, on } => (if on == "disk" { ctx.free_disk } else { ctx.free_folder }).is_some_and(|free| free < *bytes),
        T::DiskIsMounted { disk } => ctx.mounted_disks.iter().any(|d| d.eq_ignore_ascii_case(disk)),
        T::OwnedByMe => ctx.owned_by_me,
        T::Permissions { mode } => ctx.mode.is_some_and(|m| m & 0o777 == *mode),
        _ => return None,
    })
}

/// Évalue une condition d'interface : favori, récent, dossier épinglé.
pub fn eval_interface(ctx: &FileContext, test: &T) -> Option<bool> {
    Some(match test {
        T::IsFavorite => ctx.favorite,
        T::InRecents => ctx.recent,
        T::IsPinnedFolder => ctx.pinned,
        _ => return None,
    })
}
