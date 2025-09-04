//! Headless compute shader example implementation.
//!
//! This module demonstrates how to:
//! - Set up a compute shader pipeline without a window
//! - Generate a checkerboard pattern using GPU compute
//! - Read back the results and save as an image

use wgpu_engine::third_party::*;
use wgpu_engine::*;

use image::{ImageBuffer, Rgba};

/// Runs the headless compute shader example.
/// 
/// This function orchestrates the entire compute pipeline:
/// 1. Initializes wgpu state without a window
/// 2. Creates storage texture and parameter buffers
/// 3. Sets up compute shader pipeline and bind groups
/// 4. Dispatches compute work to generate a checkerboard pattern
/// 5. Reads back the texture data and saves it as "output.png"
pub fn run() -> anyhow::Result<()> {
    // Initialize wgpu state without a window (headless mode)
    let state = pollster::block_on(gfx::GfxState::new(None))?;

    // Configure the output image dimensions
    let storage_size = (1024u32, 1024u32);
    
    // Create storage texture for compute shader output
    let storage = make_storage_texture(&state, storage_size);
    
    // Create parameter buffer with checkerboard configuration
    let params = make_params_buffer(
        &state,
        ComputeParams {
            color_a: glam::vec4(0.5, 0.5, 0.5, 1.0),      // Gray color for checkerboard squares
            color_b: glam::vec4(1.0, 1.0, 1.0, 1.0),      // White color for checkerboard squares
            line_color: glam::vec4(0.0, 0.0, 0.0, 1.0),   // Black color for grid lines
            line_thickness: 3,                             // Width of grid lines in pixels
            checker_size: 128,                             // Size of each checkerboard square
        },
    );

    // Create bind groups for shader resources (texture and parameters)
    let (group_layouts, groups) = make_compute_bind_groups(&state, &storage, &params);
    let group_layouts: Vec<_> = group_layouts.iter().collect();

    // Create the compute shader pipeline
    let compute = make_compute_pipeline(&state, &group_layouts);
    
    // Create staging buffer for reading texture data back to CPU
    let staging = make_staging_buffer(&state, storage_size);

    // Create command encoder for GPU operations
    let mut encoder = state
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Compute Commands"),
        });
    
    // Record compute pass commands
    {
        let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Compute Pass"),
            timestamp_writes: None,
        });
        
        // Bind the compute pipeline and resources
        cpass.set_pipeline(&compute);
        for (i, bg) in groups.iter().enumerate() {
            cpass.set_bind_group(i as u32, bg, &[]);
        }

        // Calculate workgroup dispatch dimensions
        // Each workgroup processes an 8x8 tile of pixels
        let workgroups_x = storage_size.0.div_ceil(8);
        let workgroups_y = storage_size.1.div_ceil(8);
        cpass.dispatch_workgroups(workgroups_x, workgroups_y, 1);
    }
    // Copy the computed texture data to staging buffer for CPU readback
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: &storage,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfoBase {
            buffer: &staging,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * storage_size.0),  // 4 bytes per pixel (RGBA)
                rows_per_image: Some(storage_size.1),
            },
        },
        wgpu::Extent3d {
            width: storage_size.0,
            height: storage_size.1,
            depth_or_array_layers: 1,
        },
    );
    
    // Submit commands and wait for completion
    state.queue.submit(Some(encoder.finish()));
    state.device.poll(wgpu::PollType::Wait)?;

    // Map staging buffer for CPU access
    let buffer_slice = staging.slice(..);
    buffer_slice.map_async(wgpu::MapMode::Read, |_| {});

    // Wait for mapping to complete
    state.device.poll(wgpu::PollType::Wait)?;

    // Copy pixel data from GPU memory to CPU memory
    let data = buffer_slice.get_mapped_range();
    let bytes: Vec<u8> = data.to_vec(); // Copy into CPU memory
    drop(data);

    // Unmap the buffer to release GPU resources
    staging.unmap();

    // Create image from pixel data and save to disk
    let img: ImageBuffer<Rgba<u8>, _> =
        ImageBuffer::from_raw(storage_size.0, storage_size.1, bytes).unwrap();
    img.save("output.png")?;

    Ok(())
}

/// Parameters passed to the compute shader for checkerboard generation.
/// 
/// This struct is automatically laid out for shader compatibility using encase.
#[derive(encase::ShaderType)]
struct ComputeParams {
    /// First checkerboard square color (RGBA)
    color_a: glam::Vec4,
    /// Second checkerboard square color (RGBA)
    color_b: glam::Vec4,
    /// Grid line color (RGBA)
    line_color: glam::Vec4,
    /// Thickness of grid lines in pixels
    line_thickness: u32,
    /// Size of each checkerboard square in pixels
    checker_size: u32,
}

/// Creates a uniform buffer containing compute shader parameters.
/// 
/// Uses encase to properly layout the struct data for shader consumption.
fn make_params_buffer(state: &gfx::GfxState, params: ComputeParams) -> wgpu::Buffer {
    use wgpu::util::{self, DeviceExt};

    // Serialize parameters using shader-compatible layout
    let mut data = encase::UniformBuffer::new(Vec::<u8>::new());
    data.write(&params).unwrap();
    let bytes = data.into_inner();

    // Create GPU buffer with parameter data
    state
        .device
        .create_buffer_init(&util::BufferInitDescriptor {
            label: Some("Params"),
            contents: &bytes,
            usage: wgpu::BufferUsages::UNIFORM,
        })
}
/// Creates a staging buffer for reading texture data back to CPU.
/// 
/// The buffer is sized to hold RGBA pixel data and configured for CPU mapping.
fn make_staging_buffer(state: &gfx::GfxState, storage_size: (u32, u32)) -> wgpu::Buffer {
    let (width, height) = storage_size;
    // Calculate buffer size: 4 bytes per pixel (RGBA8)
    let buffer_size = (width * height * 4) as wgpu::BufferAddress;
    
    state.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Staging Buffer"),
        size: buffer_size,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}
/// Creates a storage texture for compute shader output.
/// 
/// The texture is configured to be written by compute shaders and copied to buffers.
fn make_storage_texture(state: &gfx::GfxState, storage_size: (u32, u32)) -> wgpu::Texture {
    let (width, height) = storage_size;
    let desc = wgpu::TextureDescriptor {
        label: Some("Shader Storage"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,  // 8-bit RGBA format
        usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    };
    state.device.create_texture(&desc)
}
/// Creates bind group layouts and bind groups for the compute shader.
/// 
/// Sets up bindings for the storage texture (output) and parameter buffer (input).
fn make_compute_bind_groups(
    state: &gfx::GfxState,
    storage: &wgpu::Texture,
    params: &wgpu::Buffer,
) -> (Vec<wgpu::BindGroupLayout>, Vec<wgpu::BindGroup>) {
    // Create bind group layout describing shader resource bindings
    let group_layout_0 = state
        .device
        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Group0 Layout"),
            entries: &[
                // Binding 0: Storage texture for compute shader output
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: wgpu::TextureFormat::Rgba8Unorm,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                // Binding 1: Uniform buffer containing shader parameters
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
    // Create texture view for storage texture binding
    let storage_view = storage.create_view(&wgpu::TextureViewDescriptor::default());
    
    // Create bind group with actual resources
    let group_0 = state.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Group0"),
        layout: &group_layout_0,
        entries: &[
            // Bind storage texture view to binding 0
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&storage_view),
            },
            // Bind parameter buffer to binding 1
            wgpu::BindGroupEntry {
                binding: 1,
                resource: params.as_entire_binding(),
            },
        ],
    });

    let groups = vec![group_0];
    let group_layouts = vec![group_layout_0];

    (group_layouts, groups)
}
/// Creates the compute shader pipeline.
/// 
/// Loads the checker.wgsl shader and creates a compute pipeline with the specified bind group layouts.
fn make_compute_pipeline(
    state: &gfx::GfxState,
    bind_group_layouts: &[&wgpu::BindGroupLayout],
) -> wgpu::ComputePipeline {
    // Load compute shader source code
    let code = include_str!("./checker.wgsl");
    let checker_module = state
        .device
        .create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("checker.wgsl"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(code)),
        });

    // Create pipeline layout with bind group layouts
    let layout = state
        .device
        .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("checker.wgsl Layout"),
            bind_group_layouts,
            push_constant_ranges: &[],
        });

    // Create the compute pipeline
    state
        .device
        .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("checker.wgsl Pipeline"),
            layout: Some(&layout),
            module: &checker_module,
            entry_point: Some("cs_main"),  // Entry function in the shader
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        })
}
