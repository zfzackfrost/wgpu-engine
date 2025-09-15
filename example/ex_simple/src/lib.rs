//! Simple triangle rendering example implementation.
//!
//! This module demonstrates basic wgpu-engine usage by:
//! - Creating a simple application client with event handling
//! - Setting up a minimal render pipeline with vertex and fragment shaders
//! - Handling mouse movement to change background color
//! - Handling keyboard input for application exit
//! - Rendering a single triangle with vertex buffers

use encase::ShaderType;
use wgpu_engine::observer::{FnSubscriber, Subscription};
use wgpu_engine::third_party::*;
use wgpu_engine::*;

pub use parking_lot::Mutex;

/// Simple application client that renders an interactive triangle.
///
/// The client manages a single render pipeline and responds to user input:
/// - Mouse movement changes the background color
/// - Escape key exits the application
struct SimpleClient {
    /// Render pipeline for drawing the triangle (protected by mutex for thread safety)
    pipeline: Mutex<Option<wgpu::RenderPipeline>>,

    mesh_index: Mutex<u8>,
    meshes: Mutex<Vec<gfx::Mesh<gfx::Vertex3D, u16>>>,

    params: Mutex<Option<gfx::UniformBuffer<GpuParams>>>,
    bind_groups: Mutex<Vec<wgpu::BindGroup>>,
    bind_group_layouts: Mutex<Vec<wgpu::BindGroupLayout>>,
}
impl std::fmt::Debug for SimpleClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimpleClient").finish_non_exhaustive()
    }
}
impl SimpleClient {
    /// Creates a new SimpleClient instance wrapped in Arc for shared ownership.
    ///
    /// The pipeline is initially None and will be initialized during the init() phase.
    #[allow(clippy::new_ret_no_self)]
    fn new() -> SharedAppClient {
        std::sync::Arc::new(Self {
            pipeline: Mutex::new(None),
            mesh_index: Mutex::new(0),
            meshes: Mutex::new(Vec::new()),
            params: Mutex::new(None),
            bind_groups: Mutex::new(Vec::new()),
            bind_group_layouts: Mutex::new(Vec::new()),
        })
    }
}

impl AppClient for SimpleClient {
    /// Initializes the client by setting up event subscriptions and creating the render pipeline.
    ///
    /// This method:
    /// 1. Subscribes to mouse movement and keyboard events
    /// 2. Creates a shader module from the embedded vertex_color.wgsl source
    /// 3. Sets up a basic render pipeline with no vertex buffers
    /// 4. Stores the pipeline for use during rendering
    /// 5. Sets up the vertex buffer and stores it for use during rendering
    fn init(&self) {
        use gfx::Vertex;
        // Get reference to this client for event handling
        let client = app_client_as::<Self>().unwrap();

        // Subscribe to mouse movement events
        {
            let client = client.clone();
            EVENTS.mouse_move().subscribe(
                FnSubscriber::new(move |data| {
                    client.handle_mouse_move(data);
                    Subscription::Keep
                })
                .boxed(),
            );
        }

        // Subscribe to keyboard events
        {
            let client = client.clone();
            EVENTS.keyboard().subscribe(
                FnSubscriber::new(move |data| {
                    client.handle_keyboard(data);
                    Subscription::Keep
                })
                .boxed(),
            );
            EVENTS.keyboard().subscribe(
                FnSubscriber::new(move |data: &KeyboardData| {
                    if data.is_pressed && data.key_code == KeyCode::F11 {
                        window::set_fullscreen(true);
                    }
                    Subscription::Keep
                })
                .boxed(),
            );
        }
        // Get application state for GPU resource creation
        let app = app();
        let mut state = app.state();
        let state = state.as_mut().unwrap();

        let mut bind_groups = self.bind_groups.lock();
        let mut bind_group_layouts = self.bind_group_layouts.lock();

        *self.params.lock() = Some(gfx::UniformBuffer::new(
            &state.device,
            &GpuParams {
                tint: glam::vec3(1.0, 1.0, 1.0),
            },
            wgpu::BufferUsages::COPY_DST,
            None,
        ));

        bind_group_layouts.push(state.device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("Group Layout 0"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            },
        ));
        bind_groups.push(state.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Group 0"),
            layout: bind_group_layouts.last().unwrap(),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: self.params.lock().as_ref().unwrap().as_entire_binding(),
            }],
        }));

        let ref_bind_group_layouts: Vec<_> = bind_group_layouts.iter().collect();

        let vertex_info = gfx::Vertex3D::info();
        // Load and create shader module from embedded WGSL source
        let module_src = include_str!("vertex_color.wgsl");
        let module = gfx::make_shader_module(
            &state.device,
            module_src,
            vertex_info.as_ref(),
            None,
            Some("vertex_color.wgsl"),
        );
        // Create pipeline layout (no bind groups or push constants needed for this simple example)
        let layout = state
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("vertex_color.wgsl Layout"),
                bind_group_layouts: &ref_bind_group_layouts,
                push_constant_ranges: &[],
            });

        // Create render pipeline with vertex and fragment shaders
        let pipeline = state
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("vertex_color.wgsl Pipeline"),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &module,
                    entry_point: Some("vs_main"),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    buffers: &[vertex_info.describe()], // One vertex buffer (Vertex3D)
                },
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw, // Counter-clockwise front faces
                    cull_mode: Some(wgpu::Face::Back), // Cull back-facing triangles
                    unclipped_depth: false,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    conservative: false,
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: gfx::Texture2D::DEPTH_FORMAT,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }), // No depth testing for this simple example
                multisample: wgpu::MultisampleState {
                    count: 1, // No multisampling
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                fragment: Some(wgpu::FragmentState {
                    module: &module,
                    entry_point: Some("fs_main"),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: state.config.clone().unwrap().format, // Match surface format
                        blend: Some(wgpu::BlendState::REPLACE),       // Replace existing colors
                        write_mask: wgpu::ColorWrites::ALL,           // Write all color channels
                    })],
                }),
                multiview: None,
                cache: None,
            });
        // Store the pipeline for use during rendering
        *self.pipeline.lock() = Some(pipeline);

        // The indices of the quad to render
        const QUAD_INDICES: &[u16] = &[
            0, 1, 2, // Triangle 0
            2, 3, 0, // Triangle 1
        ];
        let quad_indices = Some(gfx::IndexBuffer::new_filled(
            &state.device,
            QUAD_INDICES,
            wgpu::BufferUsages::empty(),
            Some("Quad Indices"),
        ));

        // The vertices of the quad to render
        let quad_vertices = &[
            // Top-right
            gfx::Vertex3D {
                position: [0.5, 0.5, 0.1],
                tex_coords: [0.0; 2], // Not used in this example
                color: [0.0, 0.0, 1.0, 1.0],
                ..Default::default()
            },
            // Top-left
            gfx::Vertex3D {
                position: [-0.5, 0.5, 0.1],
                tex_coords: [0.0; 2], // Not used in this example
                color: [1.0, 1.0, 1.0, 1.0],
                ..Default::default()
            },
            // Bottom-Left
            gfx::Vertex3D {
                position: [-0.5, -0.5, 0.1],
                tex_coords: [0.0; 2], // Not used in this example
                color: [1.0, 1.0, 0.0, 1.0],
                ..Default::default()
            },
            // Bottom-right
            gfx::Vertex3D {
                position: [0.5, -0.5, 0.1],
                tex_coords: [0.0; 2], // Not used in this example
                color: [1.0, 0.0, 0.0, 1.0],
                ..Default::default()
            },
        ];
        let mut meshes = self.meshes.lock();

        // Create the quad vertex buffer
        let quad_vertices = gfx::VertexBuffer::new_filled(
            &state.device,
            quad_vertices,
            wgpu::BufferUsages::empty(),
            Some("Quad Vertices"),
        );
        let quad = gfx::Mesh::new(quad_vertices, quad_indices);
        meshes.push(quad);

        let tri_vertices = &[
            // Top-Center
            gfx::Vertex3D {
                position: [0.0, 0.5, 0.0],
                tex_coords: [0.5, 0.0],
                color: [0.0, 0.0, 1.0, 1.0],
                ..Default::default()
            },
            // Bottom-Left
            gfx::Vertex3D {
                position: [-0.5, -0.5, 0.0],
                tex_coords: [0.0, 1.0],
                color: [1.0, 1.0, 0.0, 1.0],
                ..Default::default()
            },
            // Bottom-Right
            gfx::Vertex3D {
                position: [0.5, -0.5, 0.0],
                tex_coords: [1.0, 1.0],
                color: [1.0, 0.0, 0.0, 1.0],
                ..Default::default()
            },
        ];
        let tri_vertices = gfx::VertexBuffer::new_filled(
            &state.device,
            tri_vertices,
            wgpu::BufferUsages::empty(),
            Some("Tri Vertices"),
        );
        let tri = gfx::Mesh::new(tri_vertices, None);
        meshes.push(tri);
    }

    /// Update function called each frame (currently unused).
    fn update(&self, _delta_time: f32) {
        let app = app();
        let mut state = app.state();
        let state = state.as_mut().unwrap();

        let gray = TIME.running_time().sin() * 0.5 + 0.5;
        self.params.lock().as_ref().unwrap().write(
            &state.queue,
            0,
            &GpuParams {
                tint: glam::vec3(gray, gray, gray),
            },
        );
    }

    /// Render function that draws the triangle.
    ///
    /// Uses the stored pipeline to draw 3 vertices (forming a triangle) using a vertex
    /// buffer
    fn render(&self, rpass: &mut wgpu::RenderPass<'_>) {
        let Some(pipeline) = &*self.pipeline.lock() else {
            return;
        };
        let mesh_index = *self.mesh_index.lock() as usize;
        let meshes = self.meshes.lock();
        let meshes = if mesh_index < meshes.len() {
            &meshes[mesh_index..mesh_index + 1]
        } else {
            &meshes[..]
        };

        rpass.set_pipeline(pipeline);
        for (i, bind_group) in self.bind_groups.lock().iter().enumerate() {
            rpass.set_bind_group(i as u32, bind_group, &[]);
        }
        for mesh in meshes {
            mesh.bind(rpass);
            mesh.draw(0..1, rpass);
        }
    }
}
impl SimpleClient {
    /// Handles mouse movement events by updating the background clear color.
    ///
    /// The mouse position is normalized to [0, 1] range and used as RGB components,
    /// creating a color that changes based on cursor position.
    fn handle_mouse_move(&self, data: &MouseMoveData) {
        let app = app();
        let mut state = app.state();
        let state: &mut gfx::GfxState = state.as_mut().unwrap();
        let config = state.config.as_ref().unwrap();
        let w = config.width;
        let h = config.height;

        // Normalize mouse position to [0, 1] and use as RGB color
        state.clear_color = (data.position / glam::vec2(w as f32, h as f32))
            .extend(0.0) // Blue component set to 0
            .extend(1.0); // Alpha component set to 1 (fully opaque)

        // let delta = data.delta;
        // log::info!("Mouse Delta: ({}, {})", delta.x, delta.y);
    }

    /// Handles keyboard events, specifically the Escape key for application exit.
    fn handle_keyboard(&self, data: &KeyboardData) {
        if !data.is_pressed {
            return;
        }
        match data.key_code {
            KeyCode::Escape => {
                app().exit();
            }
            KeyCode::Space => {
                let mut mesh_index = self.mesh_index.lock();
                *mesh_index = (*mesh_index + 1) % 3;
            }
            _ => {}
        }
    }
}

#[derive(ShaderType)]
struct GpuParams {
    tint: glam::Vec3,
}

// Define the application entry point with our SimpleClient
define_entry_point!(SimpleClient::new());
