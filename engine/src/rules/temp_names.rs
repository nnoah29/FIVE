//! Noms de fichiers temporaires : jamais traités tant qu'ils portent ce nom.

use super::context::FileContext;

const SUFFIXES: [&str; 7] = [".part", ".crdownload", ".tmp", ".swp", ".swx", ".download", ".partial"];

/// Indique si le nom est celui d'un fichier temporaire (téléchargement en cours, éditeur, verrou).
pub fn is_temporary(ctx: &FileContext) -> bool {
    let name = ctx.name.to_lowercase();
    SUFFIXES.iter().any(|suffix| name.ends_with(suffix)) || name.ends_with('~') || name.starts_with(".~lock") || name.starts_with(".#")
}
