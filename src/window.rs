use std::sync::Arc;

use winit::window::{Fullscreen, Window};

use crate::app;

pub fn window() -> Option<Arc<Window>> {
    app().state().as_ref().and_then(|s| s.window.clone())
}
pub fn set_fullscreen(enable: bool) {
    let window = window().expect("No active window!");
    let fullscreen = if enable {
        Some(Fullscreen::Borderless(None))
    } else {
        None
    };
    window.set_fullscreen(fullscreen);
}
pub fn is_fullscreen() -> bool {
    window().expect("No active window!").fullscreen().is_some()
}
pub fn toggle_fullscreen() {
    set_fullscreen(!is_fullscreen());
}
