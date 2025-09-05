use std::marker::PhantomData;

#[derive(educe::Educe)]
#[educe(Deref)]
pub struct VertexBuffer<T: bytemuck::Pod + bytemuck::Zeroable> {
    #[educe(Deref)]
    buf: wgpu::Buffer,
    size: wgpu::BufferAddress,
    _data: PhantomData<T>,
}

impl<T: bytemuck::Pod + bytemuck::Zeroable> VertexBuffer<T> {
    pub fn new_filled(
        device: &wgpu::Device,
        data: &[T],
        extra_usages: wgpu::BufferUsages,
        label: Option<&str>,
    ) -> Self {
        use wgpu::util::{BufferInitDescriptor, DeviceExt};
        let bytes = bytemuck::cast_slice(data);
        let size = bytes.len() as wgpu::BufferAddress;
        let buf = device.create_buffer_init(&BufferInitDescriptor {
            label,
            contents: bytes,
            usage: extra_usages | wgpu::BufferUsages::VERTEX,
        });
        Self {
            buf,
            size,
            _data: PhantomData,
        }
    }
    pub fn size(&self) -> wgpu::BufferAddress {
        self.size
    }
    pub fn count(&self) -> u32 {
        (self.size / std::mem::size_of::<T>() as u64) as u32
    }
}
