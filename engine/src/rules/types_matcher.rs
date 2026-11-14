//! Types communs des règles : comparaison de texte, de nombres, durées, jours de la semaine.

use serde::{Deserialize, Serialize};

/// Opérateur d'une comparaison de texte.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MatchOp {
    Contains,
    Equals,
    StartsWith,
    EndsWith,
    Glob,
    Regex,
    InList,
}

/// Comparaison de texte : `value` pour un opérateur simple, `values` pour `in_list` ; casse ignorée par défaut.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextMatcher {
    pub op: MatchOp,
    #[serde(default)]
    pub value: String,
    #[serde(default)]
    pub values: Vec<String>,
    #[serde(default)]
    pub case_sensitive: bool,
}

/// Opérateur de comparaison numérique.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Cmp {
    Lt,
    Le,
    Eq,
    Ne,
    Ge,
    Gt,
}

/// Unité d'une durée.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Unit {
    Seconds,
    Minutes,
    Hours,
    Days,
    Weeks,
    Months,
}

/// Durée : un nombre et une unité (un mois vaut 30 jours).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DurationSpec {
    pub n: u64,
    pub unit: Unit,
}

/// Jour de la semaine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Weekday {
    Mon,
    Tue,
    Wed,
    Thu,
    Fri,
    Sat,
    Sun,
}
