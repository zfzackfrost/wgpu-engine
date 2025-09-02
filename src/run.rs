//! Application runner implementation
//!
//! This module contains the main `run` function that initializes logging,
//! creates the event loop, and starts the application.

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
pub use winit::event_loop::EventLoop;

use crate::app::{App, SharedAppClient, set_app};

/// Runs the application with the given client
/// 
/// This function sets up platform-specific logging, creates the winit event loop,
/// initializes the application, and starts the main event loop.
/// 
/// # Arguments
/// 
/// * `client` - The application client that defines the app's behavior
/// 
/// # Returns
/// 
/// Returns `Ok(())` on successful completion, or an error if initialization fails.
pub fn run(client: SharedAppClient) -> anyhow::Result<()> {
    // Initialize logging based on platform
    #[cfg(not(target_arch = "wasm32"))]
    {
        env_logger::init();
    }
    #[cfg(target_arch = "wasm32")]
    {
        console_log::init_with_level(log::Level::Info).unwrap_throw();
    }

    // Create the winit event loop with custom user events
    let event_loop = EventLoop::with_user_event().build()?;
    
    // Create the application from the client
    let mut app = App::from_client(
        client,
        #[cfg(target_arch = "wasm32")]
        &event_loop,
    );
    
    // Set the global application instance
    set_app(app.clone());
    
    // Start the main event loop
    event_loop.run_app(&mut app)?;
    Ok(())
}
