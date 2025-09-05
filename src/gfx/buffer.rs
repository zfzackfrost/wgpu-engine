use std::marker::PhantomData;

/// A GPU buffer specifically designed for storing vertex data.
/// 
/// This buffer wrapper provides type safety by ensuring the stored data
/// conforms to the `Pod` and `Zeroable` traits required for safe GPU memory access.
/// The buffer automatically includes `VERTEX` usage and can be extended with
/// additional usage flags as needed.
/// 
/// # Type Parameters
/// 
/// * `T` - The vertex data type, must implement `bytemuck::Pod` and `bytemuck::Zeroable`
///         for safe byte casting to GPU memory.
#[derive(educe::Educe)]
#[educe(Deref)]
pub struct VertexBuffer<T: bytemuck::Pod + bytemuck::Zeroable> {
    /// The underlying WGPU buffer that stores the vertex data
    #[educe(Deref)]
    buf: wgpu::Buffer,
    /// Total size of the buffer in bytes
    size: wgpu::BufferAddress,
    /// Phantom data to maintain type information at compile time
    _data: PhantomData<T>,
}

impl<T: bytemuck::Pod + bytemuck::Zeroable> VertexBuffer<T> {
    /// Creates a new vertex buffer filled with the provided data.
    /// 
    /// This function creates a buffer initialized with vertex data, automatically
    /// including `VERTEX` usage alongside any additional usage flags specified.
    /// The data is safely cast to bytes using `bytemuck` for GPU memory compatibility.
    /// 
    /// # Arguments
    /// 
    /// * `device` - The WGPU device used to create the buffer
    /// * `data` - Slice of vertex data to initialize the buffer with
    /// * `extra_usages` - Additional buffer usage flags beyond `VERTEX`
    /// * `label` - Optional debug label for the buffer
    /// 
    /// # Returns
    /// 
    /// A new `VertexBuffer` containing the provided vertex data
    pub fn new_filled(
        device: &wgpu::Device,
        data: &[T],
        extra_usages: wgpu::BufferUsages,
        label: Option<&str>,
    ) -> Self {
        use wgpu::util::{BufferInitDescriptor, DeviceExt};
        // Safely cast vertex data to bytes for GPU memory
        let bytes = bytemuck::cast_slice(data);
        let size = bytes.len() as wgpu::BufferAddress;
        // Create buffer with VERTEX usage plus any additional usages
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
    /// Returns the total size of the buffer in bytes.
    /// 
    /// # Returns
    /// 
    /// The buffer size as a `wgpu::BufferAddress`
    pub fn size(&self) -> wgpu::BufferAddress {
        self.size
    }
    /// Returns the number of vertices stored in this buffer.
    /// 
    /// This is calculated by dividing the total buffer size by the size
    /// of a single vertex of type `T`.
    /// 
    /// # Returns
    /// 
    /// The number of vertices as a `u32`
    pub fn count(&self) -> u32 {
        (self.size / std::mem::size_of::<T>() as u64) as u32
    }
}
