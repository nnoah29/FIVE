//! Motifs de nom : `*` (sans `/`), `**` (avec `/`), `?`, `[abc]`, `[a-z]`, `[!x]`.

/// Lit un ensemble `[...]` au début de `pattern` ; retourne (caractère accepté ?, longueur consommée) ou `None` s'il n'est pas fermé.
fn class_matches(pattern: &[char], c: char) -> Option<(bool, usize)> {
    let close = pattern.iter().skip(2).position(|x| *x == ']')? + 2;
    let (negated, body) = if pattern.get(1) == Some(&'!') { (true, &pattern[2..close]) } else { (false, &pattern[1..close]) };
    let mut found = false;
    let mut at = 0;
    while at < body.len() {
        let range = body.get(at + 1) == Some(&'-') && at + 2 < body.len();
        found |= if range { (body[at]..=body[at + 2]).contains(&c) } else { body[at] == c };
        at += if range { 3 } else { 1 };
    }
    Some((found != negated, close + 1))
}

/// Compare un motif et un texte, caractère par caractère, avec retour arrière.
fn go(pattern: &[char], text: &[char]) -> bool {
    let Some((first, rest)) = pattern.split_first() else { return text.is_empty() };
    match first {
        '*' => {
            let double = rest.first() == Some(&'*');
            let rest = if double { &rest[1..] } else { rest };
            (0..=text.len()).any(|cut| (double || !text[..cut].contains(&'/')) && go(rest, &text[cut..]))
        }
        '?' => text.first().is_some_and(|c| *c != '/') && go(rest, &text[1..]),
        '[' => match (text.first(), class_matches(pattern, text.first().copied().unwrap_or('\0'))) {
            (Some(_), Some((true, used))) => go(&pattern[used..], &text[1..]),
            (Some(c), None) => *c == '[' && go(rest, &text[1..]),
            _ => false,
        },
        c => text.first() == Some(c) && go(rest, &text[1..]),
    }
}

/// Indique si `text` correspond au motif ; la casse est ignorée sauf si `case_sensitive`.
pub fn glob_matches(pattern: &str, text: &str, case_sensitive: bool) -> bool {
    let fold = |s: &str| -> Vec<char> { if case_sensitive { s.chars().collect() } else { s.to_lowercase().chars().collect() } };
    go(&fold(pattern), &fold(text))
}
