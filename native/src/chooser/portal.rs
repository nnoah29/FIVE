//! Service du portail de bureau : FIVE répond aux demandes « ouvrir / enregistrer » des applications.

use gtk4::gio::{self, BusNameOwnerFlags, BusType, DBusConnection, DBusNodeInfo};
use gtk4::prelude::*;
use gtk4::{Application, glib};

use super::portal_call::handle_call;
use super::portal_xml::FILE_CHOOSER_XML;

const BUS_NAME: &str = "org.freedesktop.impl.portal.desktop.five";
const OBJECT_PATH: &str = "/org/freedesktop/portal/desktop";
const INTERFACE: &str = "org.freedesktop.impl.portal.FileChooser";

/// Exporte l'interface FileChooser sur la connexion et traite chaque appel.
fn register(connection: &DBusConnection) {
    let Ok(node) = DBusNodeInfo::for_xml(FILE_CHOOSER_XML) else { return };
    let Some(info) = node.lookup_interface(INTERFACE) else { return };
    let registered = connection
        .register_object(OBJECT_PATH, &info)
        .method_call(|connection, _, _, _, method, params, invocation| handle_call(&connection, method, &params, invocation))
        .build();
    if registered.is_err() {
        eprintln!("five-gtk --portal : impossible d'exporter {INTERFACE}");
    }
}

/// Lance le service : prend le nom D-Bus de FIVE et reste actif en attendant les demandes.
pub fn run() -> glib::ExitCode {
    let application = Application::builder().application_id("dev.five.files.portal").flags(gio::ApplicationFlags::NON_UNIQUE).build();
    application.connect_activate(|app| {
        std::mem::forget(app.hold());
        let quit = app.clone();
        gio::bus_own_name(BusType::Session, BUS_NAME, BusNameOwnerFlags::REPLACE, |connection, _| register(&connection), |_, _| {}, move |_, _| quit.quit());
    });
    application.run_with_args::<&str>(&[])
}
