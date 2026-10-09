use crate::data::SHARP_READER;
use ennui_quads::prelude::Quad;
use nalgebra_glm::Vec4;

pub(crate) fn reader(sharp: bool) -> u32 {
    match sharp {
        true => SHARP_READER,
        false => 0,
    }
}

pub(crate) fn raise(color: Vec4, by: f32) -> [f32; 4] {
    [
        (color.x + by).clamp(0.0, 1.0),
        (color.y + by).clamp(0.0, 1.0),
        (color.z + by).clamp(0.0, 1.0),
        color.w,
    ]
}

pub(crate) fn snapped(quad: Quad, viewport: [u32; 2]) -> Quad {
    let tall = viewport[1].max(1) as f32;
    let pixel = 2.0 / tall;
    let aspect = viewport[0].max(1) as f32 / tall;
    let snap = |value: f32, offset: f32| ((value + offset) / pixel).round() * pixel - offset;
    let [half_x, half_y] = quad.size;
    let left = snap(quad.center[0] - half_x, aspect);
    let bottom = snap(quad.center[1] - half_y, 1.0);
    let right = match half_x > 0.0 {
        true => snap(quad.center[0] + half_x, aspect).max(left + pixel),
        false => left,
    };
    let top = match half_y > 0.0 {
        true => snap(quad.center[1] + half_y, 1.0).max(bottom + pixel),
        false => bottom,
    };
    Quad {
        center: [(left + right) * 0.5, (bottom + top) * 0.5],
        size: [(right - left) * 0.5, (top - bottom) * 0.5],
        ..quad
    }
}
