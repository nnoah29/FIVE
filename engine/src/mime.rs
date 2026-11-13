//! Type MIME deviné d'après l'extension, sans lire le fichier.

const TYPES: &[(&str, &str)] = &[
    ("txt", "text/plain"),
    ("md", "text/markdown"),
    ("html", "text/html"),
    ("htm", "text/html"),
    ("css", "text/css"),
    ("js", "text/javascript"),
    ("json", "application/json"),
    ("xml", "text/xml"),
    ("csv", "text/csv"),
    ("pdf", "application/pdf"),
    ("zip", "application/zip"),
    ("tar", "application/x-tar"),
    ("gz", "application/gzip"),
    ("png", "image/png"),
    ("jpg", "image/jpeg"),
    ("jpeg", "image/jpeg"),
    ("gif", "image/gif"),
    ("webp", "image/webp"),
    ("svg", "image/svg+xml"),
    ("bmp", "image/bmp"),
    ("ico", "image/vnd.microsoft.icon"),
    ("avif", "image/avif"),
    ("mp4", "video/mp4"),
    ("webm", "video/webm"),
    ("mkv", "video/x-matroska"),
    ("mov", "video/quicktime"),
    ("avi", "video/x-msvideo"),
    ("mp3", "audio/mpeg"),
    ("flac", "audio/flac"),
    ("ogg", "audio/ogg"),
    ("wav", "audio/x-wav"),
    ("m4a", "audio/mp4"),
    ("opus", "audio/ogg"),
    ("doc", "application/msword"),
    ("rtf", "application/rtf"),
    ("docx", "application/vnd.openxmlformats-officedocument.wordprocessingml.document"),
    ("xlsx", "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"),
    ("odt", "application/vnd.oasis.opendocument.text"),
    ("sh", "application/x-sh"),
    ("py", "text/x-python"),
    ("c", "text/x-csrc"),
    ("h", "text/x-chdr"),
    ("ttf", "font/ttf"),
];

/// Type MIME d'après l'extension du nom ; chaîne vide si inconnu.
pub fn guess(name: &str) -> &'static str {
    let Some((stem, ext)) = name.rsplit_once('.') else { return "" };
    if stem.is_empty() {
        return "";
    }
    let ext = ext.to_ascii_lowercase();
    TYPES.iter().find(|(known, _)| *known == ext).map_or("", |(_, mime)| mime)
}
