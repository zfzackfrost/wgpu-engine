#[derive(educe::Educe)]
#[educe(Deref)]
pub struct Texture2D {
    #[educe(Deref)]
    tex: wgpu::Texture,
    view: wgpu::TextureView,
}
impl Texture2D {
    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
    pub fn new_attachment(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        size: (u32, u32),
        extra_usage: wgpu::TextureUsages,
        label: Option<&str>,
    ) -> Self {
        let usage = wgpu::TextureUsages::RENDER_ATTACHMENT | extra_usage;
        let tex = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size: wgpu::Extent3d {
                width: size.0,
                height: size.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage,
            view_formats: &[],
        });
        let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
        Self { tex, view }
    }
    pub fn view(&self) -> &wgpu::TextureView {
        &self.view
    }
}
