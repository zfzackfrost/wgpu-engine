use std::sync::OnceLock;

use super::SharedApp;

static APP: OnceLock<SharedApp> = OnceLock::new();
pub fn app() -> SharedApp {
    let Some(app) = APP.get() else {
        panic!("No current app!");
    };
    app.clone()
}
pub(crate) fn set_app(app: SharedApp) {
    if APP.set(app).is_err() {
        panic!("An app is already running!");
    }
}
