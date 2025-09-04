//! Headless compute shader example entry point.
//!
//! This example demonstrates running a compute shader without creating a window,
//! generating a checkerboard pattern and saving it as an image file.

use wgpu_engine::third_party::anyhow;

/// Main entry point for the headless compute shader example.
/// 
/// Executes the compute shader demo and handles any errors that occur.
fn main() -> anyhow::Result<()> {
    ex_compute_headless::run()
}
