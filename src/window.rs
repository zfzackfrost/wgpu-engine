//! Window management utilities for the wgpu-engine.
//!
//! This module provides convenient functions for managing window state,
//! particularly fullscreen control operations.

use std::sync::Arc;

use winit::window::{Fullscreen, Window};

use crate::app;

/// Retrieves the current active window from the application state.
///
/// # Returns
///
/// Returns `Some(Arc<Window>)` if a window is available, `None` otherwise.
pub fn window() -> Option<Arc<Window>> {
    // Access the application state and extract the window if available
    app().state().as_ref().and_then(|s| s.window.clone())
}

/// Sets the fullscreen mode of the active window.
///
/// # Arguments
///
/// * `enable` - If `true`, enables borderless fullscreen mode. If `false`, disables fullscreen.
///
/// # Panics
///
/// Panics if no active window is available.
pub fn set_fullscreen(enable: bool) {
    let window = window().expect("No active window!");
    // Configure fullscreen mode - borderless fullscreen on primary monitor
    let fullscreen = if enable {
        Some(Fullscreen::Borderless(None))
    } else {
        None
    };
    window.set_fullscreen(fullscreen);
}

/// Checks if the active window is currently in fullscreen mode.
///
/// # Returns
///
/// Returns `true` if the window is in fullscreen mode, `false` otherwise.
///
/// # Panics
///
/// Panics if no active window is available.
pub fn is_fullscreen() -> bool {
    window().expect("No active window!").fullscreen().is_some()
}

/// Toggles the fullscreen state of the active window.
///
/// If the window is currently in fullscreen mode, it will be switched to windowed mode.
/// If the window is in windowed mode, it will be switched to fullscreen mode.
pub fn toggle_fullscreen() {
    set_fullscreen(!is_fullscreen());
}
