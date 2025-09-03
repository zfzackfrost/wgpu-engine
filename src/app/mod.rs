//! Application module containing the main App struct and related functionality

mod client;
mod current;
mod handler;
pub use client::*;
pub use current::*;

use std::sync::Arc;

use parking_lot::{Mutex, MutexGuard};
use web_time::{Duration, Instant};

use crate::gfx::state::GfxState;

/// Main application struct that manages the application lifecycle,
/// timing, state, and client interactions
pub struct App {
    /// Event loop proxy for WASM to communicate with the event loop
    #[cfg(target_arch = "wasm32")]
    proxy: Mutex<Option<winit::event_loop::EventLoopProxy<GfxState>>>,
    /// Application state containing rendering context and window
    state: Mutex<Option<GfxState>>,
    /// Client implementation containing app-specific logic
    client: SharedAppClient,
    /// Timestamp of the last frame for delta time calculation
    last_frame_time: Mutex<Instant>,
    /// Total elapsed time since app started
    elapsed: Mutex<Duration>,
    /// Flag indicating if the app has been initialized
    is_initialized: Mutex<bool>,
    /// Flag to signal app should exit
    exit: Mutex<bool>,
}
impl App {
    /// Creates a new App instance from a client
    pub(crate) fn from_client(
        client: SharedAppClient,
        #[cfg(target_arch = "wasm32")] event_loop: &EventLoop<GfxState>,
    ) -> SharedApp {
        #[cfg(target_arch = "wasm32")]
        let proxy = Some(event_loop.create_proxy());
        let now = Instant::now();
        SharedApp(Arc::new(Self {
            state: Mutex::new(None),
            client,
            last_frame_time: Mutex::new(now),
            elapsed: Mutex::new(Duration::new(0, 0)),
            is_initialized: Mutex::new(false),
            exit: Mutex::new(false),
            #[cfg(target_arch = "wasm32")]
            proxy: Mutex::new(proxy),
        }))
    }

    /// Returns the total time the application has been running
    pub fn running_time(&self) -> Duration {
        *self.elapsed.lock()
    }
    /// Returns a lock guard to the application state
    pub fn state(&self) -> MutexGuard<'_, Option<GfxState>> {
        self.state.lock()
    }
    /// Returns a reference to the application client
    pub fn client(&self) -> Arc<dyn AppClient> {
        Arc::clone(&self.client)
    }
    /// Signals the application to exit
    pub fn exit(&self) {
        *self.exit.lock() = true;
    }
}

/// Shared reference to an App instance, allowing multiple owners
#[derive(Clone, educe::Educe)]
#[educe(Deref, DerefMut)]
pub struct SharedApp(pub Arc<App>);
