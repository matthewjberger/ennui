pub fn uniform_at(binding: u32, seen_by: wgpu::ShaderStages) -> wgpu::BindGroupLayoutEntry {
    buffer_at(binding, wgpu::BufferBindingType::Uniform, seen_by)
}

pub fn storage_at(
    slot: u32,
    writes: bool,
    seen_by: wgpu::ShaderStages,
) -> wgpu::BindGroupLayoutEntry {
    buffer_at(
        slot,
        wgpu::BufferBindingType::Storage { read_only: !writes },
        seen_by,
    )
}

fn buffer_at(
    binding: u32,
    kind: wgpu::BufferBindingType,
    seen_by: wgpu::ShaderStages,
) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: seen_by,
        ty: wgpu::BindingType::Buffer {
            ty: kind,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

pub(crate) fn texture_at(
    binding: u32,
    sample: wgpu::TextureSampleType,
    shape: wgpu::TextureViewDimension,
    seen_by: wgpu::ShaderStages,
) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: seen_by,
        ty: wgpu::BindingType::Texture {
            sample_type: sample,
            view_dimension: shape,
            multisampled: false,
        },
        count: None,
    }
}

pub(crate) fn sampler_seen_at(
    binding: u32,
    kind: wgpu::SamplerBindingType,
    seen_by: wgpu::ShaderStages,
) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: seen_by,
        ty: wgpu::BindingType::Sampler(kind),
        count: None,
    }
}

pub(crate) fn smoothed_at(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
        count: None,
    }
}

pub(crate) fn painted_at(binding: u32) -> wgpu::BindGroupLayoutEntry {
    texture_at(
        binding,
        wgpu::TextureSampleType::Float { filterable: true },
        wgpu::TextureViewDimension::D2,
        wgpu::ShaderStages::FRAGMENT,
    )
}

pub fn bind_layout(
    device: &wgpu::Device,
    label: &str,
    entries: &[wgpu::BindGroupLayoutEntry],
) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some(label),
        entries,
    })
}

pub fn binding(
    device: &wgpu::Device,
    label: &str,
    layout: &wgpu::BindGroupLayout,
    holds: Vec<wgpu::BindingResource>,
) -> wgpu::BindGroup {
    binding_at(
        device,
        label,
        layout,
        holds
            .into_iter()
            .enumerate()
            .map(|(slot, resource)| (slot as u32, resource))
            .collect(),
    )
}

pub(crate) fn binding_at(
    device: &wgpu::Device,
    label: &str,
    layout: &wgpu::BindGroupLayout,
    holds: Vec<(u32, wgpu::BindingResource)>,
) -> wgpu::BindGroup {
    let entries: Vec<wgpu::BindGroupEntry> = holds
        .into_iter()
        .map(|(binding, resource)| wgpu::BindGroupEntry { binding, resource })
        .collect();
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some(label),
        layout,
        entries: &entries,
    })
}
