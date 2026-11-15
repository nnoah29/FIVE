//! Activer ou désactiver FIVE comme sélecteur de fichiers : configuration de l'utilisateur et fichiers du service.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

use super::portal_conf::{is_enabled, set_chooser};

const PORTAL_FILE: &str = "[portal]\nDBusName=org.freedesktop.impl.portal.desktop.five\nInterfaces=org.freedesktop.impl.portal.FileChooser;\nUseIn=five\n";

/// Dossier de configuration de l'utilisateur pour les portails.
fn conf_file(home: &Path) -> PathBuf {
    home.join(".config/xdg-desktop-portal/portals.conf")
}

/// Vrai si FIVE est le sélecteur de fichiers choisi par l'utilisateur.
pub fn chooser_enabled(home: &Path) -> bool {
    fs::read_to_string(conf_file(home)).is_ok_and(|text| is_enabled(&text))
}

/// Écrit, si le paquet ne l'a pas fait, la description du service pour l'utilisateur (installation depuis les sources ou une archive).
fn ensure_service_files(home: &Path) -> io::Result<()> {
    let system = ["/usr/share", "/usr/local/share"].iter().any(|p| Path::new(p).join("xdg-desktop-portal/portals/five.portal").is_file());
    let exe = std::env::current_exe()?;
    let (portals, services) = (home.join(".local/share/xdg-desktop-portal/portals"), home.join(".local/share/dbus-1/services"));
    if system || portals.join("five.portal").is_file() {
        return Ok(());
    }
    fs::create_dir_all(&portals)?;
    fs::create_dir_all(&services)?;
    fs::write(portals.join("five.portal"), PORTAL_FILE)?;
    let service = format!("[D-BUS Service]\nName=org.freedesktop.impl.portal.desktop.five\nExec={} --portal\n", exe.display());
    fs::write(services.join("org.freedesktop.impl.portal.desktop.five.service"), service)
}

/// Active ou désactive FIVE comme sélecteur, puis relance le portail ; retourne si le portail a été relancé.
pub fn set_enabled(home: &Path, enabled: bool) -> io::Result<bool> {
    let file = conf_file(home);
    if enabled {
        ensure_service_files(home)?;
    }
    fs::create_dir_all(file.parent().unwrap_or(home))?;
    let current = fs::read_to_string(&file).unwrap_or_default();
    fs::write(&file, set_chooser(&current, enabled))?;
    Ok(Command::new("systemctl").args(["--user", "restart", "xdg-desktop-portal"]).status().is_ok_and(|s| s.success()))
}
