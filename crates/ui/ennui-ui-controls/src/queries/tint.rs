use crate::data::{Blend, Channels};
use ennui::prelude::{Entity, each_mut};
use nalgebra_glm::Vec4;

pub(crate) fn shades_in(
    channels: impl IntoIterator<Item = (Entity, usize, f32)>,
    picker: Entity,
) -> Vec4 {
    let mut held = Vec4::new(0.0, 0.0, 0.0, 1.0);
    for (band, channel, value) in channels {
        if band == picker {
            held[channel] = value;
        }
    }
    held
}

pub(crate) fn shades_of(channels: &mut Channels, picker: Entity) -> Vec4 {
    let mut found: Vec<(Entity, usize, f32)> = Vec::new();
    each_mut(channels, |_, (scrub,), (channel, band, _)| {
        found.push((band.0, channel.0, scrub.value));
    });
    shades_in(found, picker)
}

pub fn shade_of(hue: f32, saturation: f32, value: f32) -> nalgebra_glm::Vec3 {
    let held = nalgebra_glm::Vec3::new(
        ((hue + 1.0).fract() * 6.0 - 3.0).abs(),
        ((hue + 2.0 / 3.0).fract() * 6.0 - 3.0).abs(),
        ((hue + 1.0 / 3.0).fract() * 6.0 - 3.0).abs(),
    );
    nalgebra_glm::Vec3::new(
        value * (1.0 + saturation * ((held.x - 1.0).clamp(0.0, 1.0) - 1.0)),
        value * (1.0 + saturation * ((held.y - 1.0).clamp(0.0, 1.0) - 1.0)),
        value * (1.0 + saturation * ((held.z - 1.0).clamp(0.0, 1.0) - 1.0)),
    )
}

pub(crate) fn hue_of(shade: nalgebra_glm::Vec3) -> (f32, f32, f32) {
    let high = shade.x.max(shade.y).max(shade.z);
    let low = shade.x.min(shade.y).min(shade.z);
    let span = high - low;
    if span <= f32::EPSILON {
        return (0.0, 0.0, high);
    }
    let hue = match high {
        _ if high == shade.x => ((shade.y - shade.z) / span).rem_euclid(6.0),
        _ if high == shade.y => (shade.z - shade.x) / span + 2.0,
        _ => (shade.x - shade.y) / span + 4.0,
    };
    (hue / 6.0, span / high, high)
}

pub(crate) fn blended(held: Vec4, wanted: Blend) -> Vec4 {
    match wanted {
        Blend::Hsv => {
            let (hue, saturation, value) = hue_of(held.xyz());
            Vec4::new(hue, saturation, value, held.w)
        }
        Blend::Rgb => shade_of(held.x, held.y, held.z).push(held.w),
    }
}
