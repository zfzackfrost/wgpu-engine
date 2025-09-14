use wgpu_engine::third_party::*;
use wgpu_engine::*;

use image::{DynamicImage, GenericImage, GenericImageView, ImageBuffer, Rgb};

pub fn run() -> anyhow::Result<()> {
    let state = pollster::block_on(gfx::GfxState::new(None))?;

    let storage_size = (1024u32, 1024u32);

    let storage = make_storage_buffer(&state, storage_size);

    let params = make_params_buffer(
        &state,
        NoiseParams {
            width: storage_size.0,
            height: storage_size.1,
            scale: 1.0,
            octaves: 4,
            persistence: 5.5,
            lacunarity: 2.1,
            offset_x: 0.0,
            offset_y: 0.0,
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
    encoder.copy_buffer_to_buffer(&storage, 0, &staging, 0, storage.size());

    state.queue.submit(Some(encoder.finish()));
    state.device.poll(wgpu::PollType::Wait)?;

    let buffer_slice = staging.slice(..);
    buffer_slice.map_async(wgpu::MapMode::Read, |_| {});

    state.device.poll(wgpu::PollType::Wait)?;

    // Copy pixel data from GPU memory to CPU memory
    let data = buffer_slice.get_mapped_range();
    let bytes = data.to_vec();
    drop(data);

    // Unmap the buffer to release GPU resources
    staging.unmap();
    let floats: &[f32] = bytemuck::cast_slice(&bytes);
    let mut floats_rgb = Vec::with_capacity(floats.len() * 3);
    for f in floats {
        floats_rgb.push(*f);
        floats_rgb.push(*f);
        floats_rgb.push(*f);
    }

    // Create image from pixel data and save to disk
    let img: ImageBuffer<Rgb<f32>, _> =
        ImageBuffer::from_raw(storage_size.0, storage_size.1, floats_rgb).unwrap();
    let img = DynamicImage::from(img);
    img.into_rgb16().save("output.png")?;

    Ok(())
}

#[derive(encase::ShaderType)]
struct NoiseParams {
    width: u32,
    height: u32,
    scale: f32,
    octaves: u32,
    persistence: f32,
    lacunarity: f32,
    offset_x: f32,
    offset_y: f32,
}

fn make_params_buffer(state: &gfx::GfxState, params: NoiseParams) -> wgpu::Buffer {
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
    let count = storage_size.0 * storage_size.1;
    let buffer_size = (count * (std::mem::size_of::<f32>() as u32)) as u64;
    state.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Staging Buffer"),
        size: buffer_size,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}
fn make_storage_buffer(state: &gfx::GfxState, storage_size: (u32, u32)) -> wgpu::Buffer {
    let count = storage_size.0 * storage_size.1;
    let size = (count * (std::mem::size_of::<f32>() as u32)) as u64;
    state.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Compute Storage"),
        size,
        usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::STORAGE,
        mapped_at_creation: false,
    })
}
fn make_compute_bind_groups(
    state: &gfx::GfxState,
    storage: &wgpu::Buffer,
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
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
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
    let group_0 = state.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Group0"),
        layout: &group_layout_0,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: storage.as_entire_binding(),
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
    let code = include_str!("./perlin.wgsl");
    let perlin_module = state
        .device
        .create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("perlin.wgsl"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(code)),
        });

    let layout = state
        .device
        .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("perlin.wgsl Layout"),
            bind_group_layouts,
            push_constant_ranges: &[],
        });

    state
        .device
        .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("perlin.wgsl Pipeline"),
            layout: Some(&layout),
            entry_point: Some("cs_main"),
            module: &perlin_module,
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        })
}
