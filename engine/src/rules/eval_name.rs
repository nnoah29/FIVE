//! Conditions sur le nom et sur le chemin.

use super::context::FileContext;
use super::matcher::{compare, text_matches};
use super::namedate::date_in_name;
use super::types_condition::ConditionTest as T;

/// Évalue une condition de nom ; `None` si elle n'appartient pas à cette famille.
pub fn eval_name(ctx: &FileContext, test: &T) -> Option<bool> {
    Some(match test {
        T::Name { matcher } => text_matches(matcher, &ctx.name),
        T::Stem { matcher } => text_matches(matcher, &ctx.stem()),
        T::Extension { in_list } => in_list.iter().any(|ext| ext.trim_start_matches('.').eq_ignore_ascii_case(&ctx.extension)),
        T::NameLength { cmp, n } => compare(*cmp, ctx.name.chars().count() as f64, *n as f64),
        T::NameHasSpaces => ctx.name.contains(' '),
        T::NameHasUppercase => ctx.name.chars().any(char::is_uppercase),
        T::NameHasAccents => ctx.name.chars().any(|c| c.is_alphabetic() && !c.is_ascii()),
        T::NameHasDateLike => date_in_name(&ctx.name).is_some(),
        _ => return None,
    })
}

/// Indique si le fichier est dans le dossier `folder`, directement ou à n'importe quelle profondeur.
fn inside(ctx: &FileContext, folder: &str, recursive: bool) -> bool {
    let parent = ctx.path.rsplit_once('/').map_or("", |(parent, _)| parent);
    let folder = folder.trim_matches('/');
    parent == folder || (recursive && (folder.is_empty() || parent.starts_with(&format!("{folder}/"))))
}

/// Évalue une condition de chemin ; `None` si elle n'appartient pas à cette famille.
pub fn eval_path(ctx: &FileContext, test: &T) -> Option<bool> {
    Some(match test {
        T::Path { matcher } => text_matches(matcher, &ctx.path),
        T::ParentName { matcher } => text_matches(matcher, &ctx.parent_name),
        T::InFolder { path, recursive } => inside(ctx, path, *recursive),
        T::Depth { cmp, n } => compare(*cmp, ctx.depth as f64, *n as f64),
        T::InsideGitRepo => ctx.in_git,
        T::InsideTrash => ctx.in_trash,
        _ => return None,
    })
}
