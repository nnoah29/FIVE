//! Envoi d'un fichier avec prise en charge des plages d'octets (`Range`).

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use five_engine::refus::Resultat;

use crate::reply::{Reply, headers};

/// Décode `bytes=a-b`, `bytes=a-` ou `bytes=-n` en intervalle inclusif borné par `size`.
pub fn parse_range(header: Option<&str>, size: u64) -> Option<(u64, u64)> {
    let spec = header?.strip_prefix("bytes=")?;
    let (start, end) = spec.split_once('-')?;
    let (first, last) = if start.is_empty() {
        (size.saturating_sub(end.parse().ok()?), size.checked_sub(1)?)
    } else {
        (start.parse().ok()?, if end.is_empty() { size.checked_sub(1)? } else { end.parse().ok()? })
    };
    let last = last.min(size.checked_sub(1)?);
    (first <= last).then_some((first, last))
}

/// Options d'envoi : type, téléchargement éventuel, bac à sable, en-têtes en plus.
pub struct Options<'a> {
    pub kind: &'a str,
    pub disposition: Option<String>,
    pub sandbox: bool,
    pub extra: &'a [(&'a str, &'a str)],
}

/// Envoie `full` en entier ou en plage ; le contenu est lu au fil de l'eau.
pub fn send_file(full: &Path, range: Option<&str>, options: &Options) -> Resultat<Reply> {
    let mut file = File::open(full)?;
    let size = file.metadata()?.len();
    let span = parse_range(range, size);
    let (first, last) = span.unwrap_or((0, size.saturating_sub(1)));
    let length = if size == 0 { 0 } else { last - first + 1 };
    file.seek(SeekFrom::Start(first))?;
    let mut pairs: Vec<(String, String)> =
        vec![("Content-Type".into(), options.kind.into()), ("Accept-Ranges".into(), "bytes".into()), ("X-Content-Type-Options".into(), "nosniff".into())];
    if span.is_some() {
        pairs.push(("Content-Range".into(), format!("bytes {first}-{last}/{size}")));
    }
    pairs.extend(options.disposition.iter().map(|d| ("Content-Disposition".to_string(), d.clone())));
    pairs.extend(options.sandbox.then(|| ("Content-Security-Policy".to_string(), "sandbox".to_string())));
    pairs.extend(options.extra.iter().map(|(n, v)| (n.to_string(), v.to_string())));
    let view: Vec<(&str, &str)> = pairs.iter().map(|(n, v)| (n.as_str(), v.as_str())).collect();
    let reader: Box<dyn Read + Send> = Box::new(file.take(length));
    let code = if span.is_some() { 206 } else { 200 };
    Ok(Reply::new(code.into(), headers(&view), reader, Some(length as usize), None))
}
