//! Demandes du portail en cours : garde chaque sélecteur en vie et gère l'annulation par `Close`.

use std::cell::RefCell;
use std::rc::Rc;

use gtk4::gio::{DBusConnection, RegistrationId};

use super::window::Chooser;

struct Live {
    handle: String,
    chooser: Rc<Chooser>,
    registration: Option<RegistrationId>,
}

thread_local! {
    static LIVE: RefCell<Vec<Live>> = const { RefCell::new(Vec::new()) };
}

/// Mémorise une demande en cours avec l'objet `Request` exporté pour elle.
pub fn keep(handle: &str, chooser: Rc<Chooser>, registration: Option<RegistrationId>) {
    LIVE.with(|live| live.borrow_mut().push(Live { handle: handle.to_string(), chooser, registration }));
}

/// Oublie une demande terminée et retire son objet `Request` du bus.
pub fn release(connection: &DBusConnection, handle: &str) {
    let found = LIVE.with(|live| {
        let mut all = live.borrow_mut();
        all.iter().position(|l| l.handle == handle).map(|i| all.remove(i))
    });
    if let Some(Live { registration: Some(id), .. }) = found {
        let _ = connection.unregister_object(id);
    }
}

/// Annule la demande de ce `handle` (appel `Close` du portail).
pub fn close(handle: &str) {
    let chooser = LIVE.with(|live| live.borrow().iter().find(|l| l.handle == handle).map(|l| l.chooser.clone()));
    chooser.iter().for_each(|c| c.cancel());
}
