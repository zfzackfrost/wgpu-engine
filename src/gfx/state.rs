//! WGPU rendering state management
//!
//! This module contains the `State` struct which manages all WGPU resources
//! including the device, queue, surface, and rendering configuration.

use std::sync::Arc;

use winit::window::Window;

use crate::app;

/// Central rendering state that manages all WGPU resources
///
/// The State struct encapsulates the WGPU adapter, device, queue, and surface.
/// It handles initialization, resizing, and the main render loop.
pub struct State {
    /// WGPU adapter representing a physical graphics device
    pub adapter: wgpu::Adapter,
    /// WGPU logical device for creating resources
    pub device: wgpu::Device,
    /// Command queue for submitting work to the GPU
    pub queue: wgpu::Queue,
    /// Background clear color for rendering
    pub clear_color: glam::Vec4,

    /// Surface for presenting rendered frames (None for headless)
    pub surface: Option<wgpu::Surface<'static>>,
    /// Window handle (None for headless rendering)
    pub window: Option<Arc<Window>>,
    /// Surface configuration for presentation
    pub config: Option<wgpu::SurfaceConfiguration>,

    /// Internal flag tracking if surface has been configured
    pub(crate) is_surface_configured: bool,
}

impl State {
    /// Creates a new State instance, optionally with a window for presentation
    ///
    /// This function initializes all WGPU resources including the instance,
    /// adapter, device, and optionally a surface for the given window.
    ///
    /// # Arguments
    ///
    /// * `window` - Optional window for presentation. If None, creates headless state.
    ///
    /// # Returns
    ///
    /// Returns a configured State instance or an error if initialization fails.
    pub async fn new(window: Option<Arc<Window>>) -> anyhow::Result<Self> {
        let mut size = (0u32, 0u32);
        // Create WGPU instance with platform-appropriate backends
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            #[cfg(not(target_arch = "wasm32"))]
            backends: wgpu::Backends::PRIMARY, // Vulkan/Metal/DX12 on native
            #[cfg(target_arch = "wasm32")]
            backends: wgpu::Backends::GL, // WebGL on web
            ..Default::default()
        });
        // Create surface from window if provided
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

    /// Creates the initial surface configuration with appropriate format and settings
    ///
    /// # Arguments
    ///
    /// * `surface` - The surface to configure
    /// * `adapter` - The adapter to query capabilities from
    /// * `width` - Initial width in pixels
    /// * `height` - Initial height in pixels
    fn initial_surface_config(
        surface: &wgpu::Surface,
        adapter: &wgpu::Adapter,
        width: u32,
        height: u32,
    ) -> wgpu::SurfaceConfiguration {
        let surface_caps = surface.get_capabilities(adapter);
        // Prefer sRGB format for better color accuracy
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
    /// Resizes the surface to the new dimensions
    ///
    /// This function updates the surface configuration and reconfigures the surface
    /// if width and height are greater than 0.
    ///
    /// # Arguments
    ///
    /// * `width` - New width in pixels
    /// * `height` - New height in pixels
    pub fn resize(&mut self, width: u32, height: u32) {
        // Only resize if we have a surface and valid dimensions
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

    /// Executes the main render loop
    ///
    /// This function acquires the next frame, creates a render pass with the clear color,
    /// calls the application client's render method, and presents the frame.
    ///
    /// # Returns
    ///
    /// Returns Ok(()) on success, or a SurfaceError if rendering fails.
    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        // Early return if no window or surface (headless mode)
        let Some(window) = self.window.as_ref() else {
            return Ok(());
        };
        let Some(surface) = self.surface.as_ref() else {
            return Ok(());
        };

        // Request the next frame
        window.request_redraw();

        // Skip rendering if surface isn't configured yet
        if !self.is_surface_configured {
            return Ok(());
        }
        // Get the next frame to render to
        let output = surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Create command encoder for recording GPU commands
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        // Create and execute render pass
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        // Clear with the configured background color
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
            // Let the application client render its content
            app().client().render(&mut render_pass);
        }

        // Submit commands to GPU and present the frame
        self.queue.submit(Some(encoder.finish()));
        output.present();
        Ok(())
    }
}
