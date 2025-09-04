//! Simple triangle rendering example entry point.
//!
//! This example demonstrates:
//! - Basic window creation and event handling
//! - Simple render pipeline setup with a vertex and fragment shader
//! - Interactive mouse and keyboard input handling
//! - Rendering a colored triangle that responds to user input

use wgpu_engine::third_party::anyhow;

/// Main entry point for the simple triangle rendering example.
/// 
/// Initializes the application and runs the main loop with the SimpleClient.
fn main() -> anyhow::Result<()> {
    ex_simple::run()
}
