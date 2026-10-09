use crate::build::textures::{self, Shrink, Textures};
use ennui_render::resources::Images;
use naga_oil::compose::Composer;

pub struct Shading {
    pub textures: Textures,
    pub(crate) shrink: Shrink,
}

pub struct Fresh<'a> {
    pub images: &'a [usize],
    pub freed: &'a [usize],
}

pub fn build(device: &wgpu::Device, composer: &mut Composer, bindless: bool) -> Shading {
    Shading {
        textures: textures::build(device, bindless),
        shrink: textures::shrink_of(device, composer),
    }
}

pub fn fill(
    shading: &mut Shading,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    images: &Images,
    fresh: &Fresh,
) {
    textures::fill(
        &mut shading.textures,
        device,
        queue,
        images,
        fresh.images,
        fresh.freed,
        &mut shading.shrink,
    );
}
