mod client;
mod handler;
pub use client::*;

use std::sync::{Arc, OnceLock};

use parking_lot::{Mutex, MutexGuard};
use web_time::{Duration, Instant};

use crate::state::State;

pub static APP: OnceLock<SharedApp> = OnceLock::new();

pub struct App {
    #[cfg(target_arch = "wasm32")]
    proxy: Mutex<Option<winit::event_loop::EventLoopProxy<State>>>,
    state: Mutex<Option<State>>,
    client: SharedAppClient,
    last_frame_time: Mutex<Instant>,
    elapsed: Mutex<Duration>,
    is_initialized: Mutex<bool>,
    exit: Mutex<bool>,
}
impl App {
    #[allow(clippy::new_without_default)]
    #[allow(clippy::new_ret_no_self)]
    pub fn new(
        client: SharedAppClient,
        #[cfg(target_arch = "wasm32")] event_loop: &EventLoop<State>,
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

    pub fn running_time(&self) -> Duration {
        *self.elapsed.lock()
    }
    pub fn state(&self) -> MutexGuard<'_, Option<State>> {
        self.state.lock()
    }
    pub fn client(&self) -> Arc<dyn AppClient> {
        Arc::clone(&self.client)
    }
    pub fn exit(&self) {
        *self.exit.lock() = true;
    }
}

#[derive(Clone, educe::Educe)]
#[educe(Deref, DerefMut)]
pub struct SharedApp(pub Arc<App>);
