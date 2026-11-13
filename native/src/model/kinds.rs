//! Familles de fichiers : reconnaissance par extension, libellés, couleurs et icônes.

use five_engine::entry::Entry;

const KINDS: [(&str, &[&str]); 6] = [
    ("image", &["jpg", "jpeg", "png", "gif", "webp", "svg", "bmp", "avif", "ico"]),
    ("video", &["mp4", "mkv", "webm", "mov", "avi", "m4v"]),
    ("audio", &["mp3", "flac", "ogg", "wav", "m4a", "opus", "aac"]),
    ("archive", &["zip", "tar", "gz", "xz", "7z", "rar", "bz2", "zst", "deb", "iso"]),
    ("code", &["rs", "js", "ts", "py", "dart", "c", "cpp", "h", "go", "java", "sh", "html", "css", "json", "toml", "yaml", "yml", "sql", "lua", "kt"]),
    ("doc", &["pdf", "md", "txt", "doc", "docx", "odt", "rtf", "csv", "xls", "xlsx", "ods", "ppt", "pptx", "epub", "log"]),
];

/// Filtres proposés au-dessus de la liste : identifiant et libellé.
pub const FILTERS: [(&str, &str); 7] =
    [("all", "Tout"), ("image", "Images"), ("video", "Vidéos"), ("audio", "Audio"), ("doc", "Documents"), ("archive", "Archives"), ("code", "Code")];

/// Extension en minuscules, vide s'il n'y en a pas.
pub fn extension(name: &str) -> String {
    name.rsplit_once('.').map_or_else(String::new, |(_, ext)| ext.to_lowercase())
}

/// Famille d'un élément : `dir`, `image`, `video`, `audio`, `archive`, `code`, `doc` ou `other`.
pub fn kind_of(item: &Entry) -> &'static str {
    if item.is_dir {
        return "dir";
    }
    let ext = extension(&item.name);
    KINDS.iter().find(|(_, list)| list.contains(&ext.as_str())).map_or("other", |(kind, _)| kind)
}

/// Libellé français d'une famille.
pub fn kind_label(kind: &str) -> &'static str {
    match kind {
        "dir" => "Dossier",
        "image" => "Image",
        "video" => "Vidéo",
        "audio" => "Audio",
        "doc" => "Document",
        "archive" => "Archive",
        "code" => "Code",
        _ => "Autre",
    }
}

/// Nom de l'icône d'une famille.
pub fn kind_icon(kind: &str) -> &'static str {
    match kind {
        "dir" => "folder",
        "image" => "image",
        "video" => "video",
        "audio" => "audio",
        "archive" => "archive",
        "code" => "code",
        "doc" => "text",
        _ => "file",
    }
}

/// Classe CSS de la pastille de couleur d'une famille.
pub fn kind_class(kind: &str) -> &'static str {
    match kind {
        "video" => "k-video",
        "audio" => "k-audio",
        "doc" => "k-doc",
        "archive" => "k-archive",
        "code" => "k-code",
        "other" => "k-other",
        _ => "k-accent",
    }
}
