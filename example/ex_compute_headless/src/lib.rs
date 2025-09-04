use wgpu_engine::third_party::*;
use wgpu_engine::*;

use image::{ImageBuffer, Rgba};

pub fn run() -> anyhow::Result<()> {
    let state = pollster::block_on(gfx::GfxState::new(None))?;

    let storage_size = (1024u32, 1024u32);
    let storage = make_storage_texture(&state, storage_size);
    let params = make_params_buffer(
        &state,
        ComputeParams {
            color_a: glam::vec4(0.5, 0.5, 0.5, 1.0),
            color_b: glam::vec4(1.0, 1.0, 1.0, 1.0),
            line_color: glam::vec4(0.0, 0.0, 0.0, 1.0),
            line_thickness: 3,
            checker_size: 128,
        },
    );

    let (group_layouts, groups) = make_compute_bind_groups(&state, &storage, &params);
    let group_layouts: Vec<_> = group_layouts.iter().collect();

    let compute = make_compute_pipeline(&state, &group_layouts);
    let staging = make_staging_buffer(&state, storage_size);

    let mut encoder = state
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Compute Commands"),
        });
    {
        let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Compute Pass"),
            timestamp_writes: None,
        });
        cpass.set_pipeline(&compute);
        for (i, bg) in groups.iter().enumerate() {
            cpass.set_bind_group(i as u32, bg, &[]);
        }

        let workgroups_x = storage_size.0.div_ceil(8);
        let workgroups_y = storage_size.1.div_ceil(8);
        cpass.dispatch_workgroups(workgroups_x, workgroups_y, 1);
    }
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
                bytes_per_row: Some(4 * storage_size.0),
                rows_per_image: Some(storage_size.1),
            },
        },
        wgpu::Extent3d {
            width: storage_size.0,
            height: storage_size.1,
            depth_or_array_layers: 1,
        },
    );
    state.queue.submit(Some(encoder.finish()));
    state.device.poll(wgpu::PollType::Wait)?;

    let buffer_slice = staging.slice(..);
    buffer_slice.map_async(wgpu::MapMode::Read, |_| {});

    state.device.poll(wgpu::PollType::Wait)?;

    let data = buffer_slice.get_mapped_range();
    let bytes: Vec<u8> = data.to_vec(); // copy into CPU memory
    drop(data);

    staging.unmap();

    let img: ImageBuffer<Rgba<u8>, _> =
        ImageBuffer::from_raw(storage_size.0, storage_size.1, bytes).unwrap();
    img.save("output.png")?;

    Ok(())
}

#[derive(encase::ShaderType)]
struct ComputeParams {
    color_a: glam::Vec4,
    color_b: glam::Vec4,
    line_color: glam::Vec4,
    line_thickness: u32,
    checker_size: u32,
}

fn make_params_buffer(state: &gfx::GfxState, params: ComputeParams) -> wgpu::Buffer {
    use wgpu::util::{self, DeviceExt};

    let mut data = encase::UniformBuffer::new(Vec::<u8>::new());
    data.write(&params).unwrap();
    let bytes = data.into_inner();

    state
        .device
        .create_buffer_init(&util::BufferInitDescriptor {
            label: Some("Params"),
            contents: &bytes,
            usage: wgpu::BufferUsages::UNIFORM,
        })
}
fn make_staging_buffer(state: &gfx::GfxState, storage_size: (u32, u32)) -> wgpu::Buffer {
    let (width, height) = storage_size;
    let buffer_size = (width * height * 4) as wgpu::BufferAddress;
    state.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Staging Buffer"),
        size: buffer_size,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}
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
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    };
    state.device.create_texture(&desc)
}
fn make_compute_bind_groups(
    state: &gfx::GfxState,
    storage: &wgpu::Texture,
    params: &wgpu::Buffer,
) -> (Vec<wgpu::BindGroupLayout>, Vec<wgpu::BindGroup>) {
    let group_layout_0 = state
        .device
        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Group0 Layout"),
            entries: &[
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
    let storage_view = storage.create_view(&wgpu::TextureViewDescriptor::default());
    let group_0 = state.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Group0"),
        layout: &group_layout_0,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&storage_view),
            },
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
fn make_compute_pipeline(
    state: &gfx::GfxState,
    bind_group_layouts: &[&wgpu::BindGroupLayout],
) -> wgpu::ComputePipeline {
    let code = include_str!("./checker.wgsl");
    let checker_module = state
        .device
        .create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("checker.wgsl"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(code)),
        });

    let layout = state
        .device
        .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("checker.wgsl Layout"),
            bind_group_layouts,
            push_constant_ranges: &[],
        });

    state
        .device
        .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("checker.wgsl Pipeline"),
            layout: Some(&layout),
            module: &checker_module,
            entry_point: Some("cs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        })
}
