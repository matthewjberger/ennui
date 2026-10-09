use crate::commands::tint::put_channels;
use crate::components::{Band, Channel, Dial, Dot, Knob, Mixed, Scrub, Swatch, Tinted, Wheel};
use crate::data::{Blend, Channels, HSV_NAMES, Names, RGB_NAMES};
use crate::queries::tint::{blended, hue_of, shade_of, shades_in, shades_of};
use crate::theme::LEAST_VALUE;
use ennui::prelude::{Entity, Mut, Peek, Res, View, each, each_mut, peek};
use ennui_text::prelude::Label;
use ennui_ui::prelude::{
    Click, Effect, Float, Hosted, Hosts, Press, Rect, Tone, clicked, pointer_in, renew, renew_each,
};

use nalgebra_glm::{Vec2, Vec4};
use std::collections::HashMap;
use std::f32::consts::TAU;

pub(crate) fn drag_wheels(
    mut wheels: Mut<(Wheel,), (&Band, &Press, &Rect, Option<&Hosted>)>,
    mut channels: Channels,
    mut labels: Mut<(Label,)>,
    blends: Peek<Mixed>,
    hosts: Res<Hosts>,
) {
    let mut held: Vec<(Entity, f32, f32)> = Vec::new();
    each_mut(
        &mut wheels,
        |_, (mut wheel,), (band, press, rect, hosted)| {
            if !press.0 {
                return;
            }
            let at = pointer_in(&hosts, hosted);
            let away = at - rect.center;
            let radius = rect.size.x.min(rect.size.y) * 0.5;
            let hue = (away.y.atan2(away.x) / TAU).rem_euclid(1.0);
            let saturation = (away.magnitude() / radius.max(f32::EPSILON)).clamp(0.0, 1.0);
            renew(&mut wheel, Wheel { hue, saturation });
            held.push((band.0, hue, saturation));
        },
    );
    let mut shown: HashMap<Entity, Label> = HashMap::new();
    for (picker, hue, saturation) in held {
        let shades = shades_of(&mut channels, picker);
        let wanted = match peek(&blends, picker).map_or(Blend::Rgb, |held| held.0) {
            Blend::Hsv => Vec4::new(hue, saturation, shades.z, shades.w),
            Blend::Rgb => {
                let value = shades.x.max(shades.y).max(shades.z).max(LEAST_VALUE);
                shade_of(hue, saturation, value).push(shades.w)
            }
        };
        put_channels(&mut channels, &mut shown, picker, wanted);
    }
    renew_each(&mut labels, shown);
}

pub(crate) fn turn_blends(
    mut pickers: Mut<(Mixed,), (&Knob,)>,
    clicks: Peek<Click>,
    mut marks: Names,
    mut channels: Channels,
) {
    let mut turning: Vec<(Entity, Entity, Blend)> = Vec::new();
    each_mut(&mut pickers, |entity, (mut mixed,), (knob,)| {
        if !clicked(&clicks, knob.0) {
            return;
        }
        let wanted = match mixed.0 {
            Blend::Rgb => Blend::Hsv,
            Blend::Hsv => Blend::Rgb,
        };
        *mixed = Mixed(wanted);
        turning.push((entity, knob.0, wanted));
    });
    if turning.is_empty() {
        return;
    }
    let mut under: HashMap<Entity, Entity> = HashMap::new();
    let mut named: Vec<(Entity, Entity, usize)> = Vec::new();
    each_mut(&mut marks, |entity, _, (of, channel, band)| {
        if let Some(of) = of {
            under.entry(of.0).or_insert(entity);
        }
        if let (Some(channel), Some(band)) = (channel, band) {
            named.push((entity, band.0, channel.0));
        }
    });
    let mut shown: HashMap<Entity, Label> = HashMap::new();
    for (picker, button, wanted) in turning {
        let (names, text) = match wanted {
            Blend::Rgb => (RGB_NAMES, "RGB"),
            Blend::Hsv => (HSV_NAMES, "HSV"),
        };
        if let Some(mark) = under.get(&button) {
            shown.insert(*mark, Label(String::from(text)));
        }
        for (mark, band, place) in named.iter() {
            if *band == picker {
                shown.insert(*mark, Label(String::from(names[*place])));
            }
        }
        let turned = blended(shades_of(&mut channels, picker), wanted);
        put_channels(&mut channels, &mut shown, picker, turned);
    }
    renew_each(&mut marks, shown);
}

pub(crate) fn mix_tones(
    swatches: View<(&Swatch, Option<&Dial>)>,
    tinted: View<(&Tinted,)>,
    channels: View<(&Scrub, &Channel, &Band)>,
    blends: Peek<Mixed>,
    presses: Peek<Press>,
    mut tones: Mut<(Tone,)>,
    mut effects: Mut<(Effect,)>,
    mut wheels: Mut<(Wheel,)>,
) {
    let mut shades: HashMap<Entity, Tone> = HashMap::new();
    let mut lights: HashMap<Entity, Effect> = HashMap::new();
    let mut turns: HashMap<Entity, Wheel> = HashMap::new();
    for (picker, (swatch, disc)) in each(&swatches) {
        let held = shades_in(
            each(&channels).map(|(_, (scrub, channel, band))| (band.0, channel.0, scrub.value)),
            picker,
        );
        let color = match peek(&blends, picker).map_or(Blend::Rgb, |held| held.0) {
            Blend::Rgb => held,
            Blend::Hsv => shade_of(held.x, held.y, held.z).push(held.w),
        };
        shades.insert(picker, Tone::filled(color));
        shades.insert(swatch.0, Tone::filled(color));
        let Some(disc) = disc.map(|held| held.0) else {
            continue;
        };
        let (hue, saturation, value) = hue_of(color.xyz());
        lights.insert(disc, Effect([1.0, value, 0.0, 0.0]));
        if peek(&presses, disc).is_none_or(|press| !press.0) {
            turns.insert(disc, Wheel { hue, saturation });
        }
    }
    for (owner, (mixer,)) in each(&tinted) {
        if let Some(tone) = shades.get(&mixer.0).copied() {
            shades.insert(owner, tone);
        }
    }
    renew_each(&mut tones, shades);
    renew_each(&mut effects, lights);
    renew_each(&mut wheels, turns);
}

pub(crate) fn place_dots(
    wheels: View<(&Wheel, &Dot, &Rect)>,
    rects: Peek<Rect>,
    mut floats: Mut<(Float,)>,
) {
    let spots: HashMap<Entity, Float> = each(&wheels)
        .map(|(_, (wheel, dot, rect))| {
            let radius = rect.size.x.min(rect.size.y) * 0.5;
            let angle = wheel.hue * TAU;
            let away = wheel.saturation * radius;
            let middle = rect.center + Vec2::new(angle.cos() * away, angle.sin() * away);
            let size = peek(&rects, dot.0).map_or(Vec2::zeros(), |held| held.size);
            (
                dot.0,
                Float(Vec2::new(middle.x - size.x * 0.5, middle.y + size.y * 0.5)),
            )
        })
        .collect();
    renew_each(&mut floats, spots);
}
