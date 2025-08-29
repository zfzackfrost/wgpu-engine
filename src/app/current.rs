use std::sync::{Arc, OnceLock};

use super::{AppClient, SharedApp};

static APP: OnceLock<SharedApp> = OnceLock::new();
#[inline]
pub fn app() -> SharedApp {
    let Some(app) = APP.get() else {
        panic!("No current app!");
    };
    app.clone()
}
#[inline]
pub(crate) fn set_app(app: SharedApp) {
    if APP.set(app).is_err() {
        panic!("An app is already running!");
    }
}
#[inline]
pub fn app_client_as<C: AppClient>() -> Option<Arc<C>> {
    app().client().downcast_arc::<C>().ok()
}
