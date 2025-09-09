//! Application client trait definition

use std::sync::Arc;

use downcast_rs::{DowncastSync, impl_downcast};

pub struct AppClientInfo {
    pub window_title: String,
    pub window_size: glam::UVec2,
    pub wasm_canvas_selector: String,
}
impl AppClientInfo {
    #[inline]
    pub fn new() -> Self {
        Self {
            window_title: String::from("wgpu-engine"),
            window_size: glam::uvec2(1280, 720),
            wasm_canvas_selector: String::from("#wgpu-canvas"),
        }
    }
}

impl Default for AppClientInfo {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for application-specific logic that can be plugged into the main App
///
/// Implementors of this trait define custom behavior for initialization,
/// per-frame updates, and rendering.
#[allow(unused_variables)]
pub trait AppClient: DowncastSync + std::fmt::Debug {
    fn init_client_info(&self) -> AppClientInfo {
        AppClientInfo::new()
    }

    /// Called once when the application is initialized
    fn init(&self) {}
    /// Called every frame to update application logic
    ///
    /// # Arguments
    /// * `delta_time` - Time elapsed since the last frame in seconds
    fn update(&self, delta_time: f32) {}
    /// Called every frame to render application content
    ///
    /// # Arguments
    /// * `rpass` - WGPU render pass for drawing commands
    fn render(&self, rpass: &mut wgpu::RenderPass<'_>) {}
}
impl_downcast!(sync AppClient);

/// Shared reference to an AppClient implementation
pub type SharedAppClient = Arc<dyn AppClient>;
