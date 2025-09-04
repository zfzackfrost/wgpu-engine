use std::mem::size_of;

/// Trait for vertex types that can be used with the graphics pipeline.
/// 
/// Types implementing this trait must be safely transmutable to bytes (`Pod`)
/// and zero-initializable (`Zeroable`) for GPU buffer operations.
pub trait Vertex: bytemuck::Pod + bytemuck::Zeroable {
    /// Returns vertex layout information for shader binding.
    fn info() -> VertexInfoObj;
}

/// Trait for providing vertex buffer layout descriptions to wgpu.
pub trait VertexInfo {
    /// Returns the vertex buffer layout describing attribute locations and formats.
    fn describe(&self) -> wgpu::VertexBufferLayout<'_>;
}

/// Type alias for boxed vertex info objects.
pub type VertexInfoObj = Box<dyn VertexInfo>;

/// A 2D vertex with position, texture coordinates, and color.
/// 
/// Memory layout is guaranteed to match C representation for GPU compatibility.
#[repr(C)]
#[derive(Clone, Copy)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex2D {
    /// 2D position coordinates (x, y)
    pub position: [f32; 2],
    /// Texture coordinates (u, v) for sampling textures
    pub tex_coords: [f32; 2],
    /// RGBA color values, each component in range [0.0, 1.0]
    pub color: [f32; 4],
}
impl Vertex for Vertex2D {
    fn info() -> VertexInfoObj {
        struct Info;
        impl VertexInfo for Info {
            fn describe(&self) -> wgpu::VertexBufferLayout<'_> {
                // Define vertex attributes: location => format
                // 0: position (2D float), 1: tex_coords (2D float), 2: color (4D float)
                const ATTRS: &[wgpu::VertexAttribute] = &wgpu::vertex_attr_array![
                    0 => Float32x2,  // position
                    1 => Float32x2,  // tex_coords
                    2 => Float32x4,  // color
                ];
                wgpu::VertexBufferLayout {
                    array_stride: size_of::<Vertex2D>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: ATTRS,
                }
            }
        }
        Box::new(Info)
    }
}

/// A 3D vertex with position, normal, texture coordinates, and color.
/// 
/// Suitable for 3D rendering with lighting calculations using the normal vector.
/// Memory layout is guaranteed to match C representation for GPU compatibility.
#[repr(C)]
#[derive(Clone, Copy)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex3D {
    /// 3D position coordinates (x, y, z)
    pub position: [f32; 3],
    /// Surface normal vector for lighting calculations (x, y, z)
    pub normal: [f32; 3],
    /// Texture coordinates (u, v) for sampling textures
    pub tex_coords: [f32; 2],
    /// RGBA color values, each component in range [0.0, 1.0]
    pub color: [f32; 4],
}

impl Vertex for Vertex3D {
    fn info() -> VertexInfoObj {
        struct Info;
        impl VertexInfo for Info {
            fn describe(&self) -> wgpu::VertexBufferLayout<'_> {
                // Define vertex attributes: location => format
                // 0: position (3D float), 1: normal (3D float), 2: tex_coords (2D float), 3: color (4D float)
                const ATTRS: &[wgpu::VertexAttribute] = &wgpu::vertex_attr_array![
                    0 => Float32x3,  // position
                    1 => Float32x3,  // normal
                    2 => Float32x2,  // tex_coords
                    3 => Float32x4,  // color
                ];
                wgpu::VertexBufferLayout {
                    array_stride: size_of::<Vertex3D>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: ATTRS,
                }
            }
        }
        Box::new(Info)
    }
}

