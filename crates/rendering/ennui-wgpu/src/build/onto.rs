pub struct Over<'a> {
    color: &'a wgpu::TextureView,
    clear: Option<wgpu::Color>,
    stamps: Option<wgpu::RenderPassTimestampWrites<'a>>,
}

impl<'a> Over<'a> {
    pub fn onto(view: &'a wgpu::TextureView) -> Self {
        Self {
            color: view,
            clear: None,
            stamps: None,
        }
    }

    pub fn stamped(mut self, stamps: Option<wgpu::RenderPassTimestampWrites<'a>>) -> Self {
        self.stamps = stamps;
        self
    }

    pub fn clear(mut self, color: wgpu::Color) -> Self {
        self.clear = Some(color);
        self
    }
}

pub fn over<'a>(
    encoder: &'a mut wgpu::CommandEncoder,
    label: &str,
    held: Over<'a>,
) -> wgpu::RenderPass<'a> {
    encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some(label),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: held.color,
            depth_slice: None,
            resolve_target: None,
            ops: wgpu::Operations {
                load: held.clear.map_or(wgpu::LoadOp::Load, wgpu::LoadOp::Clear),
                store: wgpu::StoreOp::Store,
            },
        })],
        depth_stencil_attachment: None,
        multiview_mask: None,
        timestamp_writes: held.stamps,
        occlusion_query_set: None,
    })
}
