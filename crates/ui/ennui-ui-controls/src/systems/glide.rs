use crate::components::Glide;
use crate::queries::glide::eased;
use ennui::prelude::{Mut, Res, each_mut};
use ennui_platform::prelude::Time;
use ennui_ui::prelude::{Hidden, Pin, renew};

pub(crate) fn glide_cards(mut cards: Mut<(Glide, Pin, Hidden)>, time: Res<Time>) {
    let step = time.step;
    each_mut(&mut cards, |_, (mut glide, mut pin, mut hidden), _| {
        let toward = if glide.shown { 1.0 } else { -1.0 };
        let progress =
            (glide.progress + toward * step / glide.time.max(f32::EPSILON)).clamp(0.0, 1.0);
        let next = Glide { progress, ..*glide };
        renew(&mut glide, next);
        let share = eased(glide.ease, progress);
        let nudge = [0, 1].map(|axis| glide.rest[axis] + glide.away[axis] * (1.0 - share));
        let placed = Pin { nudge, ..*pin };
        renew(&mut pin, placed);
        renew(&mut hidden, Hidden(progress <= 0.0));
    });
}
