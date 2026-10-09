use super::bound::bind_layout;
use super::pipe::{Baking, Pipe, bake};

pub(crate) fn smooth(device: &wgpu::Device, label: &'static str) -> wgpu::Sampler {
    clamped(
        device,
        label,
        wgpu::FilterMode::Linear,
        wgpu::MipmapFilterMode::Linear,
    )
}

pub fn sharp(device: &wgpu::Device, label: &'static str) -> wgpu::Sampler {
    clamped(
        device,
        label,
        wgpu::FilterMode::Nearest,
        wgpu::MipmapFilterMode::Nearest,
    )
}

fn clamped(
    device: &wgpu::Device,
    label: &'static str,
    filter: wgpu::FilterMode,
    mipmap_filter: wgpu::MipmapFilterMode,
) -> wgpu::Sampler {
    device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some(label),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: filter,
        min_filter: filter,
        mipmap_filter,
        ..Default::default()
    })
}

pub fn screen(
    device: &wgpu::Device,
    label: &'static str,
    module: &wgpu::ShaderModule,
    entry: &'static str,
    target: wgpu::ColorTargetState,
    reads: &[wgpu::BindGroupLayoutEntry],
) -> (Baking<wgpu::RenderPipeline>, wgpu::BindGroupLayout) {
    let layout = bind_layout(device, label, reads);
    let pipeline = bake(
        Pipe::new(device, label, module)
            .layouts(&[&layout])
            .target(target)
            .fragment(entry),
    );
    (pipeline, layout)
}

pub fn sampled(
    device: &wgpu::Device,
    label: &str,
    format: wgpu::TextureFormat,
    size: [u32; 3],
    mips: u32,
    usage: wgpu::TextureUsages,
    samples: u32,
) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d {
            width: size[0].max(1),
            height: size[1].max(1),
            depth_or_array_layers: size[2].max(1),
        },
        mip_level_count: mips,
        sample_count: samples,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage,
        view_formats: &[],
    })
}

pub(crate) fn viewed(texture: &wgpu::Texture) -> wgpu::TextureView {
    texture.create_view(&wgpu::TextureViewDescriptor::default())
}
