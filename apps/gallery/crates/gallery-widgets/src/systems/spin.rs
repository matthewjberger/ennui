use crate::components::{Rank, Spun};
use crate::theme::{SPIN_APART, SPIN_BEAT, SPIN_DIMMEST};
use ennui::prelude::{Entity, Mut, Res, View, each, each_mut};
use ennui_platform::prelude::Time;
use ennui_scene::prelude::ChildOf;
use ennui_ui::prelude::{Theme, Tone};

use std::collections::HashSet;

pub(crate) fn spin_dots(
    spinners: View<(&Spun,)>,
    mut dots: Mut<(Tone,), (&Rank, &ChildOf)>,
    time: Res<Time>,
    look: Res<Theme>,
) {
    let walk = (time.since_start / SPIN_BEAT).fract();
    let held: HashSet<Entity> = each(&spinners).map(|(entity, _)| entity).collect();
    if held.is_empty() {
        return;
    }
    each_mut(&mut dots, |_, (mut tone,), (rank, of)| {
        if !held.contains(&of.0) {
            return;
        }
        let phase = (walk - rank.0 as f32 * SPIN_APART).rem_euclid(1.0);
        let share = (1.0 - (phase * 2.0 - 1.0).abs()).clamp(SPIN_DIMMEST, 1.0);
        let mut shade = look.accent;
        shade.w = share;
        let wanted = Tone::filled(shade);
        if *tone != wanted {
            *tone = wanted;
        }
    });
}
