#define_import_path ennui::screen

struct Surface {
    @builtin(position) place: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

fn corner_of(index: u32) -> Surface {
    var out: Surface;
    let uv = vec2<f32>(f32((index << 1u) & 2u), f32(index & 2u));
    out.uv = uv;
    out.place = vec4<f32>(uv * vec2<f32>(2.0, -2.0) + vec2<f32>(-1.0, 1.0), 0.0, 1.0);
    return out;
}

