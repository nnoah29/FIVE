//! Renommage par lot : validation complète, puis deux phases pour permettre les permutations.

use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

use crate::ids::unique_id;
use crate::names::clean_name;
use crate::refus::{Refus, Resultat};
use crate::root::Root;

/// Prépare le plan (source, destination) de chaque renommage en validant les noms.
fn plan(root: &Root, pairs: &[(String, String)]) -> Resultat<Vec<(PathBuf, PathBuf)>> {
    let mut out = Vec::new();
    for (rel, new) in pairs {
        let full = root.resolve_entry(rel)?;
        if root.in_trash(&full) {
            return Err(Refus::new(400, "Impossible de renommer dans la corbeille"));
        }
        let folder = full.parent().map(PathBuf::from).unwrap_or_default();
        out.push((full, folder.join(clean_name(new)?)));
    }
    Ok(out)
}

/// Refuse les doublons de destination et les écrasements d'éléments étrangers au lot.
fn check_conflicts(plan: &[(PathBuf, PathBuf)]) -> Resultat<()> {
    let sources: HashSet<&PathBuf> = plan.iter().map(|(src, _)| src).collect();
    let targets: HashSet<&PathBuf> = plan.iter().map(|(_, dst)| dst).collect();
    if targets.len() != plan.len() {
        return Err(Refus::new(409, "Deux éléments auraient le même nom"));
    }
    for (src, dst) in plan {
        if src != dst && fs::symlink_metadata(dst).is_ok() && !sources.contains(dst) {
            let name = dst.file_name().map_or_else(String::new, |n| n.to_string_lossy().into_owned());
            return Err(Refus::new(409, format!("« {name} » existe déjà")));
        }
    }
    Ok(())
}

/// Renomme tous les éléments ou aucun ; retourne le nombre d'éléments réellement renommés.
pub fn rename_many(root: &Root, pairs: &[(String, String)]) -> Resultat<usize> {
    let plan = plan(root, pairs)?;
    check_conflicts(&plan)?;
    let mut staged = Vec::new();
    for (src, dst) in plan.iter().filter(|(src, dst)| src != dst) {
        let folder = src.parent().map(PathBuf::from).unwrap_or_default();
        let temporary = folder.join(format!(".five-{}", unique_id()));
        fs::rename(src, &temporary)?;
        staged.push((temporary, dst));
    }
    for (temporary, dst) in &staged {
        fs::rename(temporary, dst)?;
    }
    Ok(staged.len())
}
