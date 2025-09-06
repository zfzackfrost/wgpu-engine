use bytemuck::{Pod, Zeroable};

use super::BytemuckBuffer;

pub trait IndexType: Pod + Zeroable {
    fn index_format() -> wgpu::IndexFormat;
}
impl IndexType for u16 {
    fn index_format() -> wgpu::IndexFormat {
        wgpu::IndexFormat::Uint16
    }
}
impl IndexType for u32 {
    fn index_format() -> wgpu::IndexFormat {
        wgpu::IndexFormat::Uint32
    }
}

#[derive(educe::Educe)]
#[educe(Deref)]
pub struct IndexBuffer<T: IndexType>(BytemuckBuffer<T>);

impl<T: IndexType> IndexBuffer<T> {
    pub fn new(
        device: &wgpu::Device,
        count: u64,
        extra_usage: wgpu::BufferUsages,
        label: Option<&str>,
    ) -> Self {
        Self(BytemuckBuffer::new(
            device,
            count,
            extra_usage | wgpu::BufferUsages::INDEX,
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
            extra_usages | wgpu::BufferUsages::INDEX,
            label,
        ))
    }
    pub fn index_format(&self) -> wgpu::IndexFormat {
        T::index_format()
    }
}
