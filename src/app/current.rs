//! Global application instance management
//!
//! This module provides functions to set and access the currently running
//! application instance globally.

use std::sync::{Arc, OnceLock};

use super::{AppClient, SharedApp};

// Global application instance storage
static APP: OnceLock<SharedApp> = OnceLock::new();
/// Returns the current global application instance
/// 
/// # Panics
/// Panics if no application has been set via `set_app()`
#[inline]
pub fn app() -> SharedApp {
    let Some(app) = APP.get() else {
        panic!("No current app!");
    };
    app.clone()
}
/// Sets the global application instance
/// 
/// # Panics
/// Panics if an application has already been set
#[inline]
pub(crate) fn set_app(app: SharedApp) {
    if APP.set(app).is_err() {
        panic!("An app is already running!");
    }
}
/// Attempts to downcast the current app's client to a specific type
/// 
/// # Returns
/// `Some(Arc<C>)` if the client is of type C, `None` otherwise
#[inline]
pub fn app_client_as<C: AppClient>() -> Option<Arc<C>> {
    app().client().downcast_arc::<C>().ok()
}
