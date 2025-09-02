//! Application client trait definition

use std::sync::Arc;

use downcast_rs::{DowncastSync, impl_downcast};

/// Trait for application-specific logic that can be plugged into the main App
/// 
/// Implementors of this trait define custom behavior for initialization,
/// per-frame updates, and rendering.
#[allow(unused_variables)]
pub trait AppClient: DowncastSync + std::fmt::Debug {
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
