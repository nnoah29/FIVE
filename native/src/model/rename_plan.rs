//! Plan de renommage par lot : remplacement ou modèle, avec vérification des conflits.

use std::collections::HashSet;

use regex::RegexBuilder;

use super::paths::name_parts;

/// Réglages du renommage par lot.
#[derive(Debug, Clone)]
pub struct RenameSpec {
    pub template_mode: bool,
    pub find: String,
    pub replacement: String,
    pub regex: bool,
    pub ignore_case: bool,
    pub template: String,
    pub start: u64,
    pub pad: usize,
}

impl Default for RenameSpec {
    /// Réglages initiaux : remplacement sans casse, modèle `{name}-{n}{ext}`.
    fn default() -> Self {
        Self {
            template_mode: false,
            find: String::new(),
            replacement: String::new(),
            regex: false,
            ignore_case: true,
            template: "{name}-{n}{ext}".into(),
            start: 1,
            pad: 2,
        }
    }
}

/// Nouveau nom d'un élément selon le modèle, à la position `index` du lot.
fn from_template(spec: &RenameSpec, name: &str, index: usize) -> String {
    let (stem, tail) = name_parts(name);
    let number = format!("{:0>width$}", spec.start + index as u64, width = spec.pad);
    spec.template.replace("{name}", &stem).replace("{ext}", &tail).replace("{n}", &number)
}

/// Nouveau nom par remplacement ; `Err` décrit une expression invalide.
fn from_replace(spec: &RenameSpec, name: &str) -> Result<String, String> {
    if spec.find.is_empty() {
        return Ok(name.to_string());
    }
    let source = if spec.regex { spec.find.clone() } else { regex::escape(&spec.find) };
    let pattern =
        RegexBuilder::new(&source).case_insensitive(spec.ignore_case).build().map_err(|e| e.to_string().lines().last().unwrap_or("").trim().to_string())?;
    Ok(pattern.replace_all(name, spec.replacement.replace("$&", "${0}").as_str()).into_owned())
}

/// Nouveau nom de l'élément `name`, à la position `index` du lot.
pub fn new_name(spec: &RenameSpec, name: &str, index: usize) -> Result<String, String> {
    if spec.template_mode { Ok(from_template(spec, name, index)) } else { from_replace(spec, name) }
}

/// Problème de chaque nouveau nom : vide si tout va bien.
pub fn check_names(olds: &[String], news: &[String], siblings: &[String]) -> Vec<String> {
    let own: HashSet<&String> = olds.iter().collect();
    let others: HashSet<&String> = siblings.iter().filter(|n| !own.contains(n)).collect();
    news.iter()
        .enumerate()
        .map(|(at, name)| {
            if name.trim().is_empty() || name.contains('/') || name == "." || name == ".." {
                "Nom invalide"
            } else if news.iter().position(|n| n == name) != Some(at) && *name != olds[at] {
                "Doublon dans le lot"
            } else if others.contains(name) {
                "Existe déjà"
            } else {
                ""
            }
            .to_string()
        })
        .collect()
}
