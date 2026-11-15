//! Un appel du portail : lit la demande, affiche le sélecteur, renvoie le résultat à l'application.

use std::path::PathBuf;

use gtk4::gio::{DBusConnection, DBusMethodInvocation, DBusNodeInfo};
use gtk4::glib::{Variant, VariantDict};
use gtk4::prelude::*;

use super::live;
use super::options::{open_mode, parse_request, path_from_bytes};
use super::outcome::{Choice, results_variant};
use super::portal_xml::REQUEST_XML;
use super::request::{Mode, Request};
use super::window::Chooser;

/// Texte d'un argument de l'appel (`app_id`, `title`…).
fn text_arg(params: &Variant, index: usize) -> String {
    params.child_value(index).str().map(String::from).unwrap_or_default()
}

/// Construit la demande selon la méthode appelée ; `None` si la méthode est inconnue.
fn request_for(method: &str, title: &str, options: &Variant) -> Option<Request> {
    match method {
        "OpenFile" => Some(parse_request(open_mode(options), title, options)),
        "SaveFile" => Some(parse_request(Mode::Save, title, options)),
        "SaveFiles" => Some(parse_request(Mode::Folder, title, options)),
        _ => None,
    }
}

/// Pour « enregistrer plusieurs fichiers » : chaque nom demandé est placé dans le dossier choisi.
fn with_file_names(choice: Choice, options: &Variant) -> Choice {
    let dict = VariantDict::new(Some(options));
    let names: Vec<Vec<u8>> = dict.lookup::<Vec<Vec<u8>>>("files").ok().flatten().unwrap_or_default();
    let dir = choice.paths.first().cloned().unwrap_or_default();
    Choice { paths: names.iter().map(|n| dir.join(path_from_bytes(n))).collect::<Vec<PathBuf>>(), filter: choice.filter }
}

/// Répond à l'appel : 0 avec les adresses choisies, 1 si l'utilisateur a annulé.
fn reply(invocation: DBusMethodInvocation, request: &Request, choice: Option<Choice>) {
    let (code, results) = match choice {
        Some(choice) => (0u32, results_variant(request, &choice)),
        None => (1u32, VariantDict::new(None).end()),
    };
    invocation.return_value(Some(&Variant::tuple_from_iter([code.to_variant(), results])));
}

/// Exporte l'objet `Request` du `handle` pour que le portail puisse annuler la demande.
fn export_request(connection: &DBusConnection, handle: &str) -> Option<gtk4::gio::RegistrationId> {
    let node = DBusNodeInfo::for_xml(REQUEST_XML).ok()?;
    let info = node.lookup_interface("org.freedesktop.impl.portal.Request")?;
    let handle_text = handle.to_string();
    connection
        .register_object(handle, &info)
        .method_call(move |_, _, _, _, _, _, invocation| {
            live::close(&handle_text);
            invocation.return_value(None);
        })
        .build()
        .ok()
}

/// Traite un appel `OpenFile`, `SaveFile` ou `SaveFiles`.
pub fn handle_call(connection: &DBusConnection, method: &str, params: &Variant, invocation: DBusMethodInvocation) {
    let (handle, options) = (text_arg(params, 0), params.child_value(4));
    let Some(request) = request_for(method, &text_arg(params, 3), &options) else {
        invocation.return_dbus_error("org.freedesktop.DBus.Error.UnknownMethod", "méthode inconnue");
        return;
    };
    let (connection_done, handle_done, request_done) = (connection.clone(), handle.clone(), request.clone());
    let many = method == "SaveFiles";
    let chooser = Chooser::show(
        request,
        Box::new(move |choice| {
            live::release(&connection_done, &handle_done);
            reply(invocation, &request_done, choice.map(|c| if many { with_file_names(c, &options) } else { c }));
        }),
    );
    live::keep(&handle, chooser, export_request(connection, &handle));
}
