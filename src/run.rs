#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
pub use winit::event_loop::EventLoop;

use crate::app::{APP, App, SharedAppClient};
pub fn run(client: SharedAppClient) -> anyhow::Result<()> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        env_logger::init();
    }
    #[cfg(target_arch = "wasm32")]
    {
        console_log::init_with_level(log::Level::Info).unwrap_throw();
    }

    let event_loop = EventLoop::with_user_event().build()?;
    let mut app = App::from_client(
        client,
        #[cfg(target_arch = "wasm32")]
        &event_loop,
    );
    assert!(APP.set(app.clone()).is_ok(), "APP was already initialized!");
    event_loop.run_app(&mut app)?;
    Ok(())
}
