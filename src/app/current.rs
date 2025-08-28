use std::sync::OnceLock;

use super::SharedApp;

pub(crate) static APP: OnceLock<SharedApp> = OnceLock::new();
pub fn app() -> SharedApp {
    let Some(app) = APP.get() else {
        panic!("No current app!");
    };
    app.clone()
}
