//! Tri des éléments : dossiers d'abord, puis nom naturel, type, taille ou date.

use std::cmp::Ordering;

use five_engine::entry::Entry;

use super::kinds::kind_of;

/// Colonne de tri.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortKey {
    Name,
    Kind,
    Modified,
    Size,
}

impl SortKey {
    /// Lit la lettre enregistrée dans les réglages (`n`, `t`, `m`, `s`).
    pub fn from_letter(letter: &str) -> SortKey {
        match letter {
            "t" => SortKey::Kind,
            "m" => SortKey::Modified,
            "s" => SortKey::Size,
            _ => SortKey::Name,
        }
    }

    /// Lettre enregistrée dans les réglages.
    pub fn letter(self) -> &'static str {
        match self {
            SortKey::Name => "n",
            SortKey::Kind => "t",
            SortKey::Modified => "m",
            SortKey::Size => "s",
        }
    }
}

/// Lettre sans accent ni majuscule, pour comparer comme le fait la langue française.
fn fold(ch: char) -> char {
    match ch.to_lowercase().next().unwrap_or(ch) {
        'à' | 'â' | 'ä' => 'a',
        'ç' => 'c',
        'é' | 'è' | 'ê' | 'ë' => 'e',
        'î' | 'ï' => 'i',
        'ô' | 'ö' => 'o',
        'ù' | 'û' | 'ü' => 'u',
        other => other,
    }
}

/// Découpe un nom en blocs de chiffres et de non-chiffres.
fn chunks(text: &str) -> Vec<(bool, String)> {
    let mut out: Vec<(bool, String)> = Vec::new();
    for ch in text.chars() {
        let digit = ch.is_ascii_digit();
        match out.last_mut() {
            Some((kind, part)) if *kind == digit => part.push(if digit { ch } else { fold(ch) }),
            _ => out.push((digit, if digit { ch.to_string() } else { fold(ch).to_string() })),
        }
    }
    out
}

/// Compare deux noms : chiffres par valeur, lettres sans tenir compte de la casse ni des accents.
pub fn natural_cmp(a: &str, b: &str) -> Ordering {
    let (left, right) = (chunks(a), chunks(b));
    for (x, y) in left.iter().zip(&right) {
        let order = match (x.0, y.0) {
            (true, true) => {
                x.1.trim_start_matches('0')
                    .len()
                    .cmp(&y.1.trim_start_matches('0').len())
                    .then_with(|| x.1.trim_start_matches('0').cmp(y.1.trim_start_matches('0')))
            }
            _ => x.1.cmp(&y.1),
        };
        if order != Ordering::Equal {
            return order;
        }
    }
    left.len().cmp(&right.len())
}

/// Trie une liste : dossiers avant fichiers, puis selon la colonne et le sens demandés.
pub fn sort_items(list: &[Entry], key: SortKey, ascending: bool) -> Vec<Entry> {
    let mut out = list.to_vec();
    out.sort_by(|a, b| {
        let by_kind = b.is_dir.cmp(&a.is_dir);
        let order = match key {
            SortKey::Name => natural_cmp(&a.name, &b.name),
            SortKey::Kind => kind_of(a).cmp(kind_of(b)).then_with(|| natural_cmp(&a.name, &b.name)),
            SortKey::Modified => a.modified.partial_cmp(&b.modified).unwrap_or(Ordering::Equal),
            SortKey::Size => a.size.cmp(&b.size),
        };
        by_kind.then(if ascending { order } else { order.reverse() })
    });
    out
}
