#import ennui::screen::{Surface, corner_of}

@group(0) @binding(0) var finer: texture_2d<f32>;
@group(0) @binding(1) var reader: sampler;

@vertex
fn vertex_main(@builtin(vertex_index) corner: u32) -> Surface {
    return corner_of(corner);
}

@fragment
fn fragment_main(surface: Surface) -> @location(0) vec4<f32> {
    return textureSampleLevel(finer, reader, surface.uv, 0.0);
}
