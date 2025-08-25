use std::sync::Arc;

use downcast_rs::{DowncastSync, impl_downcast};

pub trait AppClient: DowncastSync + std::fmt::Debug {
    fn init(&self) {}
    fn update(&self, delta_time: f32) {}
    fn render(&self, rpass: &mut wgpu::RenderPass<'_>) {}
}
impl_downcast!(sync AppClient);
pub type SharedAppClient = Arc<dyn AppClient>;
