pub mod readback;
pub mod stopwatch;
pub mod textures;

use crate::guest::Bench;
use naga_oil::compose::Composer;

mod bound;
mod onto;
mod paint;
mod pipe;
pub mod room;

pub use bound::*;
pub use onto::*;
pub use paint::*;
pub use pipe::*;
pub use room::*;

pub fn shader(bench: &mut Bench, label: &str, source: &str) -> wgpu::ShaderModule {
    composed(
        bench.device,
        &mut *bench.composer,
        label,
        source,
        textures::bindless(&bench.shading.textures),
    )
}

pub(crate) fn composed(
    device: &wgpu::Device,
    composer: &mut Composer,
    label: &str,
    source: &str,
    bindless: bool,
) -> wgpu::ShaderModule {
    device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(label),
        source: wgpu::ShaderSource::Naga(std::borrow::Cow::Owned(crate::compose::module(
            composer, label, source, bindless,
        ))),
    })
}
