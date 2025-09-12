use std::marker::PhantomData;

#[derive(educe::Educe)]
#[educe(Deref)]
pub struct BytemuckBuffer<T: bytemuck::Pod + bytemuck::Zeroable> {
    #[educe(Deref)]
    buf: wgpu::Buffer,
    label: Option<String>,
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
            label: label.map(String::from),
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
            label: label.map(String::from),
            _data: PhantomData,
        }
    }
    pub fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }
    pub fn write(&self, queue: &wgpu::Queue, offset: wgpu::BufferAddress, data: &[T]) {
        let data = bytemuck::cast_slice(data);
        let requested_sz = data.len() as u64;
        let sz = self.size();
        assert!(
            (requested_sz + offset) <= sz,
            "Requested an out-of-bounds write for buffer: {}",
            self.label().unwrap_or("<NO NAME>")
        );
        queue.write_buffer(&self.buf, offset, data);
    }
    pub fn count(&self) -> u32 {
        (self.size() / std::mem::size_of::<T>() as wgpu::BufferAddress) as u32
    }
}
