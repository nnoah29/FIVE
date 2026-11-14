//! Conditions sur le contenu : texte, image, audio et vidéo, archive.

use super::context::FileContext;
use super::matcher::{compare, text_matches};
use super::types_condition::ConditionTest as T;

/// Évalue une condition de texte (200 Ko lus au plus) ; un contenu non lu rend la condition fausse.
pub fn eval_text(ctx: &FileContext, test: &T) -> Option<bool> {
    let text = ctx.text.as_deref();
    Some(match test {
        T::TextContains { matcher } => text.is_some_and(|t| text_matches(matcher, t)),
        T::LineCount { cmp, n } => text.is_some_and(|t| compare(*cmp, t.lines().count() as f64, *n as f64)),
        T::FirstLine { matcher } => text.is_some_and(|t| text_matches(matcher, t.lines().next().unwrap_or(""))),
        T::HasShebang => text.is_some_and(|t| t.starts_with("#!")),
        T::Encoding { value } => ctx.encoding.as_deref().is_some_and(|e| e.eq_ignore_ascii_case(value)),
        _ => return None,
    })
}

/// Évalue une condition d'image : dimensions, orientation, mégapixels, transparence.
pub fn eval_image(ctx: &FileContext, test: &T) -> Option<bool> {
    let image = ctx.image.as_ref();
    Some(match test {
        T::ImageWidth { cmp, px } => image.is_some_and(|i| compare(*cmp, i.width as f64, *px as f64)),
        T::ImageHeight { cmp, px } => image.is_some_and(|i| compare(*cmp, i.height as f64, *px as f64)),
        T::Orientation { value } => image.is_some_and(|i| match value.as_str() {
            "landscape" => i.width > i.height,
            "portrait" => i.height > i.width,
            "square" => i.width == i.height,
            _ => false,
        }),
        T::Megapixels { cmp, n } => image.is_some_and(|i| compare(*cmp, (i.width * i.height) as f64 / 1_000_000.0, *n)),
        T::HasAlpha => image.is_some_and(|i| i.has_alpha),
        _ => return None,
    })
}

/// Évalue une condition d'audio ou de vidéo : durée, résolution, piste son.
pub fn eval_media(ctx: &FileContext, test: &T) -> Option<bool> {
    let media = ctx.media.as_ref();
    Some(match test {
        T::Duration { cmp, seconds } => media.is_some_and(|m| compare(*cmp, m.duration, *seconds)),
        T::VideoResolution { cmp, px } => media.is_some_and(|m| compare(*cmp, m.height as f64, *px as f64)),
        T::HasAudioTrack => media.is_some_and(|m| m.has_audio),
        _ => return None,
    })
}

/// Évalue une condition d'archive : lisible, nombre d'entrées, nom contenu, chiffrée.
pub fn eval_archive(ctx: &FileContext, test: &T) -> Option<bool> {
    let archive = ctx.archive.as_ref();
    Some(match test {
        T::ArchiveIsReadable => archive.is_some_and(|a| a.readable),
        T::ArchiveEntryCount { cmp, n } => archive.is_some_and(|a| compare(*cmp, a.entries.len() as f64, *n as f64)),
        T::ArchiveContainsName { matcher } => archive.is_some_and(|a| a.entries.iter().any(|e| text_matches(matcher, e))),
        T::ArchiveIsEncrypted => archive.is_some_and(|a| a.encrypted),
        _ => return None,
    })
}
