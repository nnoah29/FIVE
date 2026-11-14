//! Comparaisons : texte (contient, égal, glob, regex, liste) et nombres.

use std::cell::RefCell;
use std::collections::HashMap;

use regex::{Regex, RegexBuilder};

use super::glob::glob_matches;
use super::types_matcher::{Cmp, MatchOp, TextMatcher};

thread_local! {
    static CACHE: RefCell<HashMap<(String, bool), Option<Regex>>> = RefCell::new(HashMap::new());
}

/// Compare `text` à une expression régulière compilée une fois (taille bornée) ; une expression invalide ne correspond jamais.
fn regex_matches(pattern: &str, text: &str, case_sensitive: bool) -> bool {
    CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        if cache.len() > 256 {
            cache.clear();
        }
        let compiled = cache
            .entry((pattern.to_string(), case_sensitive))
            .or_insert_with(|| RegexBuilder::new(pattern).case_insensitive(!case_sensitive).size_limit(1 << 20).build().ok());
        compiled.as_ref().is_some_and(|re| re.is_match(text))
    })
}

/// Indique si le texte satisfait la comparaison (casse ignorée par défaut).
pub fn text_matches(matcher: &TextMatcher, text: &str) -> bool {
    let cs = matcher.case_sensitive;
    let fold = |s: &str| if cs { s.to_string() } else { s.to_lowercase() };
    let (text_f, value) = (fold(text), fold(&matcher.value));
    match matcher.op {
        MatchOp::Contains => text_f.contains(&value),
        MatchOp::Equals => text_f == value,
        MatchOp::StartsWith => text_f.starts_with(&value),
        MatchOp::EndsWith => text_f.ends_with(&value),
        MatchOp::Glob => glob_matches(&matcher.value, text, cs),
        MatchOp::Regex => regex_matches(&matcher.value, text, cs),
        MatchOp::InList => matcher.values.iter().any(|item| fold(item) == text_f),
    }
}

/// Compare deux nombres selon l'opérateur.
pub fn compare(cmp: Cmp, value: f64, reference: f64) -> bool {
    match cmp {
        Cmp::Lt => value < reference,
        Cmp::Le => value <= reference,
        Cmp::Eq => value == reference,
        Cmp::Ne => value != reference,
        Cmp::Ge => value >= reference,
        Cmp::Gt => value > reference,
    }
}
