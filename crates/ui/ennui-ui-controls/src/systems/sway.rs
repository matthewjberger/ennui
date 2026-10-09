use crate::data::Swings;
use crate::theme::KNOB_PAD;
use ennui::prelude::{Entity, Mut, Peek, Res, each_mut, peek};
use ennui_platform::prelude::Time;
use ennui_ui::prelude::{Float, Hosted, Hosts, Rect, Theme, Tone, renew_each, toward, worn_theme};
use nalgebra_glm::Vec2;
use std::collections::HashMap;

pub(crate) fn sway_toggles(
    mut swings: Swings,
    mut floats: Mut<(Float,)>,
    mut tones: Mut<(Tone,)>,
    rects: Peek<Rect>,
    placed: Peek<Hosted>,
    themes: Peek<Theme>,
    time: Res<Time>,
    look: Res<Theme>,
    hosts: Res<Hosts>,
) {
    let step = time.since_last_frame;
    let (enter, leave) = (look.enter, look.leave);
    let mut held: Vec<(Entity, Entity, f32)> = Vec::new();
    each_mut(&mut swings, |_, (mut swing,), (toggle, knob, track)| {
        if let Some(toggle) = toggle {
            swing.0 = toward(swing.0, f32::from(toggle.0), step, enter, leave);
        }
        if let (Some(knob), Some(track)) = (knob, track) {
            held.push((knob.0, track.0, swing.0));
        }
    });
    let pad = KNOB_PAD;
    let mut spots: HashMap<Entity, Float> = HashMap::new();
    let mut shades: HashMap<Entity, Tone> = HashMap::new();
    for (knob, track, swing) in held {
        let Some(rect) = peek(&rects, track).copied() else {
            continue;
        };
        let size = peek(&rects, knob).map_or(Vec2::zeros(), |held| held.size);
        let travel = (rect.size.x - size.x - pad * 2.0).max(0.0);
        spots.insert(
            knob,
            Float(Vec2::new(
                rect.center.x - rect.size.x * 0.5 + pad + travel * swing,
                rect.center.y + size.y * 0.5,
            )),
        );
        let worn = worn_theme(&themes, &look, &hosts, peek(&placed, track));
        let shade = worn.ground + (worn.accent - worn.ground) * swing.clamp(0.0, 1.0);
        shades.insert(track, Tone::filled(shade));
    }
    renew_each(&mut floats, spots);
    renew_each(&mut tones, shades);
}
