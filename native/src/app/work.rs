//! Travail bloquant hors du fil de l'interface, résultat rendu sur le fil de l'interface.

use gtk4::{gio, glib};

/// Exécute `job` dans un thread de travail puis appelle `done` avec son résultat, sur le fil de l'interface.
pub fn spawn_work<T: Send + 'static>(job: impl FnOnce() -> T + Send + 'static, done: impl FnOnce(T) + 'static) {
    glib::spawn_future_local(async move {
        if let Ok(value) = gio::spawn_blocking(job).await {
            done(value);
        }
    });
}
