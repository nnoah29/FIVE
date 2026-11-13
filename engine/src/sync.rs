//! Verrou tolérant à l'empoisonnement : un thread en panne ne bloque pas les autres.

use std::sync::{Mutex, MutexGuard, PoisonError};

/// Prend le verrou même si un thread précédent a paniqué en le tenant.
pub fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}
