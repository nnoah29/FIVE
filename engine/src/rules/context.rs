//! Ce que l'évaluateur sait d'un fichier : état courant et informations demandées à la demande.

use crate::entry::Entry;

use super::kinds::kind_of_extension;

/// Dimensions et transparence d'une image.
#[derive(Debug, Clone, Default)]
pub struct ImageInfo {
    pub width: u64,
    pub height: u64,
    pub has_alpha: bool,
}

/// Durée, hauteur d'image et son d'un fichier audio ou vidéo.
#[derive(Debug, Clone, Default)]
pub struct MediaInfo {
    pub duration: f64,
    pub height: u64,
    pub has_audio: bool,
}

/// Archive lue sans extraction : lisible, chiffrée, noms des entrées.
#[derive(Debug, Clone, Default)]
pub struct ArchiveInfo {
    pub readable: bool,
    pub encrypted: bool,
    pub entries: Vec<String>,
}

/// Doublons d'un fichier dans une portée : autres fichiers au contenu identique, rang par date, homonymes.
#[derive(Debug, Clone, Default)]
pub struct DuplicateGroup {
    pub others: u32,
    pub newest: bool,
    pub oldest: bool,
    pub same_name: u32,
}

/// Doublons selon la portée : dossier, dossiers surveillés, racine.
#[derive(Debug, Clone, Default)]
pub struct Duplicates {
    pub folder: DuplicateGroup,
    pub watched: DuplicateGroup,
    pub root: DuplicateGroup,
}

/// Un fichier ou dossier à évaluer ; les champs facultatifs sont remplis par l'appelant seulement si une condition les demande.
#[derive(Debug, Clone, Default)]
pub struct FileContext {
    pub path: String,
    pub name: String,
    pub parent_name: String,
    pub extension: String,
    pub kind: String,
    pub depth: usize,
    pub is_dir: bool,
    pub is_symlink: bool,
    pub broken_symlink: bool,
    pub executable: bool,
    pub read_only: bool,
    pub empty: bool,
    pub mount_point: bool,
    pub size: u64,
    pub modified: Option<f64>,
    pub created: Option<f64>,
    pub accessed: Option<f64>,
    pub now: f64,
    pub in_git: bool,
    pub in_trash: bool,
    pub child_count: Option<u64>,
    pub file_count: Option<u64>,
    pub folder_size: Option<u64>,
    pub mime: Option<String>,
    pub text: Option<String>,
    pub encoding: Option<String>,
    pub hash: Option<String>,
    pub image: Option<ImageInfo>,
    pub media: Option<MediaInfo>,
    pub archive: Option<ArchiveInfo>,
    pub duplicates: Option<Duplicates>,
    pub git_status: Option<String>,
    pub origin_url: Option<String>,
    pub disk: Option<String>,
    pub mounted_disks: Vec<String>,
    pub free_folder: Option<u64>,
    pub free_disk: Option<u64>,
    pub owned_by_me: bool,
    pub mode: Option<u32>,
    pub favorite: bool,
    pub recent: bool,
    pub pinned: bool,
    pub stable_for: Option<f64>,
    pub counter: u32,
}

impl FileContext {
    /// Contexte minimal d'un fichier d'après son chemin relatif : nom, extension, famille, profondeur, dossier parent.
    pub fn new(path: &str) -> FileContext {
        let name = path.rsplit('/').next().unwrap_or("").to_string();
        let parent = path.rsplit_once('/').map_or("", |(parent, _)| parent);
        let extension = name.rsplit_once('.').filter(|(stem, _)| !stem.is_empty()).map_or(String::new(), |(_, ext)| ext.to_lowercase());
        let kind = kind_of_extension(&extension).to_string();
        let parent_name = parent.rsplit('/').next().unwrap_or("").to_string();
        FileContext { path: path.to_string(), name, parent_name, extension, kind, depth: path.matches('/').count(), ..FileContext::default() }
    }

    /// Contexte d'un élément déjà décrit par le moteur.
    pub fn from_entry(entry: &Entry, now: f64) -> FileContext {
        let mut context = FileContext::new(&entry.path);
        (context.is_dir, context.is_symlink, context.size, context.modified, context.now) =
            (entry.is_dir, entry.is_link, entry.size, Some(entry.modified), now);
        context.mime = entry.mime.clone();
        context.empty = if entry.is_dir { entry.children == Some(Some(0)) } else { entry.size == 0 };
        context
    }

    /// Nom sans extension.
    pub fn stem(&self) -> String {
        match self.name.rsplit_once('.') {
            Some((stem, _)) if !stem.is_empty() => stem.to_string(),
            _ => self.name.clone(),
        }
    }

    /// Indique si le nom commence par un point.
    pub fn hidden(&self) -> bool {
        self.name.starts_with('.')
    }
}
