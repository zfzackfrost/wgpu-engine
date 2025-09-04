//! Simple triangle rendering example implementation.
//!
//! This module demonstrates basic wgpu-engine usage by:
//! - Creating a simple application client with event handling
//! - Setting up a minimal render pipeline with vertex and fragment shaders
//! - Handling mouse movement to change background color
//! - Handling keyboard input for application exit
//! - Rendering a single triangle without vertex buffers (using vertex ID)

use wgpu_engine::observer::FnSubscriber;
pub use wgpu_engine::third_party::*;
pub use wgpu_engine::*;

pub use parking_lot::Mutex;

/// Simple application client that renders an interactive triangle.
/// 
/// The client manages a single render pipeline and responds to user input:
/// - Mouse movement changes the background color
/// - Escape key exits the application
#[derive(educe::Educe)]
#[educe(Debug)]
struct SimpleClient {
    /// Render pipeline for drawing the triangle (protected by mutex for thread safety)
    #[educe(Debug(ignore))]
    pipeline: Mutex<Option<wgpu::RenderPipeline>>,
}
impl SimpleClient {
    /// Creates a new SimpleClient instance wrapped in Arc for shared ownership.
    /// 
    /// The pipeline is initially None and will be initialized during the init() phase.
    #[allow(clippy::new_ret_no_self)]
    fn new() -> SharedAppClient {
        std::sync::Arc::new(Self {
            pipeline: Mutex::new(None),
        })
    }
}

impl AppClient for SimpleClient {
    /// Initializes the client by setting up event subscriptions and creating the render pipeline.
    /// 
    /// This method:
    /// 1. Subscribes to mouse movement and keyboard events
    /// 2. Creates a shader module from the embedded triangle.wgsl source
    /// 3. Sets up a basic render pipeline with no vertex buffers
    /// 4. Stores the pipeline for use during rendering
    fn init(&self) {
        // Get reference to this client for event handling
        let client = app_client_as::<Self>().unwrap();
        
        // Subscribe to mouse movement events
        {
            let client = client.clone();
            EVENTS.mouse_move().subscribe(
                FnSubscriber::new(move |data| {
                    client.handle_mouse_move(data);
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
                })
                .boxed(),
            );
        }
        // Get application state for GPU resource creation
        let app = app();
        let mut state = app.state();
        let state = state.as_mut().unwrap();

        // Load and create shader module from embedded WGSL source
        let module_src = include_str!("triangle.wgsl");
        let module = state
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("triangle.wgsl"),
                source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(module_src)),
            });
        // Create pipeline layout (no bind groups or push constants needed for this simple example)
        let layout = state
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("triangle.wgsl Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });
        // Create render pipeline with vertex and fragment shaders
        let pipeline = state
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("triangle.wgsl Pipeline"),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &module,
                    entry_point: Some("vs_main"),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    buffers: &[], // No vertex buffers - using vertex ID in shader
                },
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,           // Counter-clockwise front faces
                    cull_mode: Some(wgpu::Face::Back),          // Cull back-facing triangles
                    unclipped_depth: false,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    conservative: false,
                },
                depth_stencil: None, // No depth testing for this simple example
                multisample: wgpu::MultisampleState {
                    count: 1,                                   // No multisampling
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                fragment: Some(wgpu::FragmentState {
                    module: &module,
                    entry_point: Some("fs_main"),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: state.config.clone().unwrap().format, // Match surface format
                        blend: Some(wgpu::BlendState::REPLACE),         // Replace existing colors
                        write_mask: wgpu::ColorWrites::ALL,             // Write all color channels
                    })],
                }),
                multiview: None,
                cache: None,
            });
        // Store the pipeline for use during rendering
        *self.pipeline.lock() = Some(pipeline);
    }

    /// Update function called each frame (currently unused).
    fn update(&self, _delta_time: f32) {}

    /// Render function that draws the triangle.
    /// 
    /// Uses the stored pipeline to draw 3 vertices (forming a triangle)
    /// without any vertex buffers - the vertex positions are generated
    /// in the vertex shader using the vertex index.
    fn render(&self, rpass: &mut wgpu::RenderPass<'_>) {
        let Some(pipeline) = &*self.pipeline.lock() else {
            return;
        };
        rpass.set_pipeline(pipeline);
        rpass.draw(0..3, 0..1); // Draw 3 vertices, 1 instance
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
            .extend(0.0)  // Blue component set to 0
            .extend(1.0); // Alpha component set to 1 (fully opaque)
        
        let delta = data.delta;
        log::info!("Mouse Delta: ({}, {})", delta.x, delta.y);
    }

    /// Handles keyboard events, specifically the Escape key for application exit.
    fn handle_keyboard(&self, data: &KeyboardData) {
        if data.is_pressed && data.key_code == KeyCode::Escape {
            app().exit();
        }
    }
}

// Define the application entry point with our SimpleClient
define_entry_point!(SimpleClient::new());
