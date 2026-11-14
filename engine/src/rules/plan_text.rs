//! Texte calculé d'une action : nouveau nom pour un renommage, dossier ou message pour les autres.

use super::context::FileContext;
use super::namedate::date_in_name;
use super::rename_ops::{change_case, normalize_extension, replace_in_name, slugify, split_name};
use super::template::expand;
use super::time_calc::format_date;
use super::types_action::{DateSource, Position, RuleAction as A};

/// Indique si l'action change le nom du fichier.
pub fn is_rename(action: &A) -> bool {
    matches!(
        action,
        A::Rename { .. }
            | A::ReplaceInName { .. }
            | A::ChangeCase { .. }
            | A::Slugify
            | A::AddPrefix { .. }
            | A::AddSuffix { .. }
            | A::AddDate { .. }
            | A::AddCounter { .. }
            | A::RemoveSpaces
            | A::NormalizeExtension
    )
}

/// Date écrite dans un nom ou une date de fichier, mise en forme ; une date absente est une erreur.
fn dated(ctx: &FileContext, from: DateSource, format: &str) -> Result<String, String> {
    let missing = || "date indisponible pour ce fichier".to_string();
    match from {
        DateSource::Modified => ctx.modified.map(|t| format_date(t, format)).ok_or_else(missing),
        DateSource::Created => ctx.created.map(|t| format_date(t, format)).ok_or_else(missing),
        DateSource::Today => Ok(format_date(ctx.now, format)),
        DateSource::Name => date_in_name(&ctx.name)
            .map(|(y, m, d)| format.replace("AAAA", &format!("{y:04}")).replace("MM", &format!("{m:02}")).replace("JJ", &format!("{d:02}")))
            .ok_or_else(missing),
    }
}

/// Nouveau nom produit par une action de renommage à partir du nom courant.
pub fn new_name(action: &A, ctx: &FileContext, current: &str) -> Result<String, String> {
    let (stem, ext) = split_name(current);
    Ok(match action {
        A::Rename { template } => expand(template, ctx)?,
        A::ReplaceInName { find, replace, regex } => replace_in_name(current, find, replace, *regex),
        A::ChangeCase { mode } => change_case(current, *mode),
        A::Slugify => slugify(current),
        A::AddPrefix { text } => expand(text, ctx)? + current,
        A::AddSuffix { text } => format!("{stem}{}{ext}", expand(text, ctx)?),
        A::AddDate { from, position, format } => match position {
            Position::Prefix => format!("{} {current}", dated(ctx, *from, format)?),
            Position::Suffix => format!("{stem} {}{ext}", dated(ctx, *from, format)?),
        },
        A::AddCounter { start, width } => format!("{stem}_{:0w$}{ext}", *start as u64 + ctx.counter as u64, w = *width as usize),
        A::RemoveSpaces => stem.replace(' ', "") + &ext,
        A::NormalizeExtension => normalize_extension(current),
        _ => current.to_string(),
    })
}

/// Libellé français du dossier d'une famille de fichiers.
fn kind_label(kind: &str) -> &'static str {
    match kind {
        "image" => "Images",
        "video" => "Vidéos",
        "audio" => "Audio",
        "document" => "Documents",
        "archive" => "Archives",
        "code" => "Code",
        _ => "Autres",
    }
}

/// Joint deux morceaux de chemin relatif.
fn join(base: &str, tail: &str) -> String {
    let base = base.trim_matches('/');
    if base.is_empty() { tail.to_string() } else { format!("{base}/{tail}") }
}

/// Texte calculé d'une action qui n'est pas un renommage : dossier de destination, nom d'archive, message, date.
pub fn target_text(action: &A, ctx: &FileContext) -> Result<Option<String>, String> {
    Ok(match action {
        A::MoveTo { folder } | A::CopyTo { folder } => Some(expand(folder, ctx)?),
        A::MoveToDated { folder, pattern } => Some(join(&expand(folder, ctx)?, &expand(pattern, ctx)?)),
        A::MoveToByKind { base } => Some(join(&expand(base, ctx)?, kind_label(&ctx.kind))),
        A::MoveToByExtension { base } => Some(join(&expand(base, ctx)?, if ctx.extension.is_empty() { "sans-extension" } else { &ctx.extension })),
        A::CreateFolder { template } => Some(expand(template, ctx)?),
        A::Zip { name_template: Some(template), .. } => Some(expand(template, ctx)?),
        A::Notify { message_template } => Some(expand(message_template, ctx)?),
        A::SetModifiedTime { from } => Some(dated(ctx, *from, "AAAA-MM-JJ hh:mm")?),
        _ => None,
    })
}
