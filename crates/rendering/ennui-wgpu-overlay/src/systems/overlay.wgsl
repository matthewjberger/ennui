#import ennui::quad::{Quad, Placed, NO_PICTURE, quad_placed, quad_color, quad_spots}

struct Fragment {
    @builtin(position) place: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) local: vec2<f32>,
    @location(2) at: vec2<f32>,
    @location(3) @interpolate(flat) index: u32,
}

#ifdef BINDLESS
@group(0) @binding(0) var pictures: binding_array<texture_2d<f32>>;
@group(0) @binding(1) var readers: binding_array<sampler>;
#else
@group(0) @binding(0) var picture: texture_2d<f32>;
@group(2) @binding(0) var reader: sampler;
#endif
@group(1) @binding(0) var<uniform> display: vec4<f32>;
@group(1) @binding(1) var<storage, read> quads: array<Quad>;

fn encoded(linear: vec3<f32>) -> vec3<f32> {
    let low = linear * 12.92;
    let high = 1.055 * pow(max(linear, vec3<f32>(0.0)), vec3<f32>(1.0 / 2.4)) - 0.055;
    return select(high, low, linear <= vec3<f32>(0.0031308));
}

fn seen(quad: Quad, at: vec2<f32>) -> vec4<f32> {
#ifdef BINDLESS
    return textureSampleLevel(pictures[quad.picture[0]], readers[quad.picture[1]], at, 0.0);
#else
    return textureSampleLevel(picture, reader, at, 0.0);
#endif
}

@vertex
fn vertex_main(@builtin(instance_index) instance: u32, @builtin(vertex_index) corner: u32) -> Fragment {
    let placed = quad_placed(quads[instance], corner);
    var fragment: Fragment;
    fragment.place = vec4<f32>(placed.at.x * display.y, placed.at.y, 0.0, 1.0);
    fragment.uv = placed.uv;
    fragment.local = placed.local;
    fragment.at = placed.at;
    fragment.index = instance;
    return fragment;
}

@fragment
fn fragment_main(fragment: Fragment) -> @location(0) vec4<f32> {
    let quad = quads[fragment.index];
    let across = dpdx(fragment.uv);
    let down = dpdy(fragment.uv);
    var held = vec4<f32>(1.0);
    var spread = vec4<f32>(1.0);
    if quad.picture[0] != NO_PICTURE {
        held = seen(quad, fragment.uv);
        spread = vec4<f32>(held.a);
        if quad.shape.z > 0.5 {
            let spots = quad_spots(fragment.uv, across, down);
            spread = vec4<f32>(
                seen(quad, spots[0]).a,
                seen(quad, spots[1]).a,
                seen(quad, spots[2]).a,
                seen(quad, spots[3]).a,
            );
        }
    }
    let color = quad_color(quad, Placed(fragment.at, fragment.local, fragment.uv), held, spread);
    let lit = color.rgb * display.x;
    if display.z > 0.5 {
        return vec4<f32>(encoded(lit), color.a);
    }
    return vec4<f32>(lit, color.a);
}
