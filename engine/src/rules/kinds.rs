//! Famille d'un fichier d'après son extension : image, vidéo, audio, document, archive, code ou autre.

const FAMILIES: [(&str, &[&str]); 6] = [
    ("image", &["png", "jpg", "jpeg", "gif", "webp", "svg", "bmp", "ico", "avif", "tif", "tiff", "heic", "raw"]),
    ("video", &["mp4", "mkv", "webm", "mov", "avi", "wmv", "flv", "m4v"]),
    ("audio", &["mp3", "flac", "ogg", "wav", "m4a", "opus", "aac", "wma"]),
    ("document", &["pdf", "doc", "docx", "odt", "rtf", "txt", "md", "xls", "xlsx", "ods", "csv", "ppt", "pptx", "odp", "epub"]),
    ("archive", &["zip", "tar", "gz", "tgz", "xz", "7z", "rar", "bz2", "zst", "deb", "iso", "jar"]),
    ("code", &["rs", "py", "js", "ts", "c", "h", "cpp", "java", "go", "sh", "html", "css", "json", "toml", "yaml", "yml", "xml", "sql", "php", "rb"]),
];

/// Famille d'une extension (en minuscules, sans point) ; `other` si elle est inconnue.
pub fn kind_of_extension(extension: &str) -> &'static str {
    FAMILIES.iter().find(|(_, list)| list.contains(&extension)).map_or("other", |(name, _)| name)
}
