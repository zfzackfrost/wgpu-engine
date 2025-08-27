use wgpu_engine::observer::FnSubscriber;
pub use wgpu_engine::third_party::*;
pub use wgpu_engine::*;

pub use parking_lot::Mutex;

#[derive(educe::Educe)]
#[educe(Debug)]
struct SimpleClient {
    #[educe(Debug(ignore))]
    pipeline: Mutex<Option<wgpu::RenderPipeline>>,
}
impl SimpleClient {
    #[allow(clippy::new_ret_no_self)]
    fn new() -> SharedAppClient {
        std::sync::Arc::new(Self {
            pipeline: Mutex::new(None),
        })
    }
}

impl AppClient for SimpleClient {
    fn init(&self) {
        let Some(app) = APP.get() else {
            panic!("No active app!");
        };
        EVENTS.mouse_move().subscribe(
            FnSubscriber::new(|data| {
                let client = app.client().downcast_arc::<SimpleClient>().unwrap();
                client.handle_mouse_move(data);
            })
            .boxed(),
        );
        EVENTS.keyboard().subscribe(
            FnSubscriber::new(|data| {
                let client = app.client().downcast_arc::<SimpleClient>().unwrap();
                client.handle_keyboard(data);
            })
            .boxed(),
        );
        let mut state = app.state();
        let state = state.as_mut().unwrap();

        let module_src = include_str!("triangle.wgsl");
        let module = state
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("triangle.wgsl"),
                source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(module_src)),
            });
        let layout = state
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("triangle.wgsl Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });
        let pipeline = state
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("triangle.wgsl Pipeline"),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &module,
                    entry_point: Some("vs_main"),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    buffers: &[],
                },
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    unclipped_depth: false,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                fragment: Some(wgpu::FragmentState {
                    module: &module,
                    entry_point: Some("fs_main"),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: state.config.clone().unwrap().format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview: None,
                cache: None,
            });
        *self.pipeline.lock() = Some(pipeline);
    }
    fn update(&self, _delta_time: f32) {}
    fn render(&self, rpass: &mut wgpu::RenderPass<'_>) {
        let Some(pipeline) = &*self.pipeline.lock() else {
            return;
        };
        rpass.set_pipeline(pipeline);
        rpass.draw(0..3, 0..1);
    }
}
impl SimpleClient {
    fn handle_mouse_move(&self, data: &MouseMoveData) {
        let Some(app) = APP.get() else {
            return;
        };
        let mut state = app.state();
        let state: &mut wgpu_engine::State = state.as_mut().unwrap();
        let config = state.config.as_ref().unwrap();
        let w = config.width;
        let h = config.height;
        state.clear_color = (data.position / glam::vec2(w as f32, h as f32))
            .extend(0.0)
            .extend(1.0);
    }
    fn handle_keyboard(&self, data: &KeyboardData) {
        if data.is_pressed && data.key_code == KeyCode::Escape {
            let Some(app) = APP.get() else {
                return;
            };
            app.exit();
        }
    }
}

define_entry_point!(SimpleClient::new());
