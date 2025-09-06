use bytemuck::{Pod, Zeroable};

use super::BytemuckBuffer;

#[derive(educe::Educe)]
#[educe(Deref)]
pub struct VertexBuffer<T: Pod + Zeroable>(BytemuckBuffer<T>);

impl<T: Pod + Zeroable> VertexBuffer<T> {
    pub fn new(
        device: &wgpu::Device,
        count: u64,
        extra_usage: wgpu::BufferUsages,
        label: Option<&str>,
    ) -> Self {
        Self(BytemuckBuffer::new(
            device,
            count,
            extra_usage | wgpu::BufferUsages::VERTEX,
            label,
        ))
    }
    pub fn new_filled(
        device: &wgpu::Device,
        data: &[T],
        extra_usages: wgpu::BufferUsages,
        label: Option<&str>,
    ) -> Self {
        Self(BytemuckBuffer::new_filled(
            device,
            data,
            extra_usages | wgpu::BufferUsages::VERTEX,
            label,
        ))
    }
}
