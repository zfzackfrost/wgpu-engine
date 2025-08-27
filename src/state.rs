use std::sync::Arc;

use winit::window::Window;

use crate::APP;

pub struct State {
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub clear_color: glam::Vec4,

    pub surface: Option<wgpu::Surface<'static>>,
    pub window: Option<Arc<Window>>,
    pub config: Option<wgpu::SurfaceConfiguration>,

    pub(crate) is_surface_configured: bool,
}

impl State {
    pub async fn new(window: Option<Arc<Window>>) -> anyhow::Result<Self> {
        let mut size = (0u32, 0u32);
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            #[cfg(not(target_arch = "wasm32"))]
            backends: wgpu::Backends::PRIMARY,
            #[cfg(target_arch = "wasm32")]
            backends: wgpu::Backends::GL,
            ..Default::default()
        });
        let surface = window.clone().map(|w| {
            let s = w.inner_size();
            size.0 = s.width;
            size.1 = s.height;
            instance.create_surface(w.clone()).unwrap()
        });
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: surface.as_ref(),
            })
            .await?;
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
                },
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
            })
            .await?;
        let config = surface
            .as_ref()
            .map(|surface| Self::initial_surface_config(surface, &adapter, size.0, size.1));
        Ok(Self {
            adapter,
            device,
            queue,
            surface,
            window,
            is_surface_configured: false,
            config,
            clear_color: glam::vec4(0.0, 0.0, 0.0, 1.0),
        })
    }

    fn initial_surface_config(
        surface: &wgpu::Surface,
        adapter: &wgpu::Adapter,
        width: u32,
        height: u32,
    ) -> wgpu::SurfaceConfiguration {
        let surface_caps = surface.get_capabilities(adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width,
            height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        }
    }
    pub fn resize(&mut self, width: u32, height: u32) {
        if let Some(surface) = self.surface.as_mut()
            && let Some(config) = self.config.as_mut()
            && width > 0
            && height > 0
        {
            config.width = width;
            config.height = height;
            surface.configure(&self.device, self.config.as_ref().unwrap());
            self.is_surface_configured = true;
        }
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let Some(window) = self.window.as_ref() else {
            return Ok(());
        };
        let Some(surface) = self.surface.as_ref() else {
            return Ok(());
        };

        window.request_redraw();

        if !self.is_surface_configured {
            return Ok(());
        }
        let output = surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: self.clear_color.x as f64,
                            g: self.clear_color.y as f64,
                            b: self.clear_color.z as f64,
                            a: self.clear_color.w as f64,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            let Some(app) = APP.get() else {
                panic!("No active app!");
            };
            app.client().render(&mut render_pass);
        }
        self.queue.submit(Some(encoder.finish()));
        output.present();
        Ok(())
    }
}
