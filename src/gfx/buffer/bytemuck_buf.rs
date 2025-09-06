use std::marker::PhantomData;

#[derive(educe::Educe)]
#[educe(Deref)]
pub struct BytemuckBuffer<T: bytemuck::Pod + bytemuck::Zeroable> {
    #[educe(Deref)]
    buf: wgpu::Buffer,
    _data: PhantomData<T>,
}

impl<T: bytemuck::Pod + bytemuck::Zeroable> BytemuckBuffer<T> {
    pub fn new(
        device: &wgpu::Device,
        count: u64,
        usage: wgpu::BufferUsages,
        label: Option<&str>,
    ) -> Self {
        let buf = device.create_buffer(&wgpu::BufferDescriptor {
            label,
            size: count as wgpu::BufferAddress * std::mem::size_of::<T>() as wgpu::BufferAddress,
            usage,
            mapped_at_creation: false,
        });
        Self {
            buf,
            _data: PhantomData,
        }
    }
    pub fn new_filled(
        device: &wgpu::Device,
        data: &[T],
        usage: wgpu::BufferUsages,
        label: Option<&str>,
    ) -> Self {
        use wgpu::util::{BufferInitDescriptor, DeviceExt};
        let contents = bytemuck::cast_slice(data);
        let buf = device.create_buffer_init(&BufferInitDescriptor {
            label,
            contents,
            usage,
        });
        Self {
            buf,
            _data: PhantomData,
        }
    }
    pub fn write(&self, queue: &wgpu::Queue, offset: wgpu::BufferAddress, data: &[T]) {
        let data = bytemuck::cast_slice(data);
        queue.write_buffer(&self.buf, offset, data);
    }
    pub fn count(&self) -> u32 {
        (self.size() / std::mem::size_of::<T>() as wgpu::BufferAddress) as u32
    }
}
