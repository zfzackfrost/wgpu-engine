use std::marker::PhantomData;

#[derive(educe::Educe)]
#[educe(Deref)]
pub struct UniformBuffer<T: encase::ShaderType + encase::internal::WriteInto> {
    #[educe(Deref)]
    buf: wgpu::Buffer,
    _data: PhantomData<T>,
}

impl<T: encase::ShaderType + encase::internal::WriteInto> UniformBuffer<T> {
    pub fn new(
        device: &wgpu::Device,
        data: &T,
        extra_usage: wgpu::BufferUsages,
        label: Option<&str>,
    ) -> Self {
        use wgpu::util::{BufferInitDescriptor, DeviceExt};
        let mut buffer_writer = encase::UniformBuffer::new(Vec::<u8>::new());
        buffer_writer.write(data).unwrap();
        let buffer_data = buffer_writer.into_inner();
        let buf = device.create_buffer_init(&BufferInitDescriptor {
            label,
            contents: &buffer_data,
            usage: extra_usage | wgpu::BufferUsages::UNIFORM,
        });
        Self {
            buf,
            _data: Default::default(),
        }
    }
    pub fn write(&self, queue: &wgpu::Queue, offset: wgpu::BufferAddress, data: &T) {
        let mut buffer_writer = encase::UniformBuffer::new(Vec::<u8>::new());
        buffer_writer.write(data).unwrap();
        let buffer_data = buffer_writer.into_inner();
        queue.write_buffer(&self.buf, offset, &buffer_data);
    }
}
