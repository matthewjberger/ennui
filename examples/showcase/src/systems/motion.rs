use crate::resources::{Clock, Motion};
use crate::theme::{
    BACK_SWING, BAR_DELAY, BAR_SHARES, BAR_STAGGER, BAR_TIME, BAR_TRACK, LONGEST_STEP, PULSE_SPEED,
    SLIDE_TIME, SLIDE_WIDTH, STAMP_DELAY, STAMP_TIME,
};
use ennui::prelude::{Res, ResMut};
use ennui_platform::prelude::Time;
use ennui_ui::prelude::Span;

pub fn restart(mut clock: ResMut<Clock>) {
    clock.since = 0.0;
}

pub fn play(time: Res<Time>, mut clock: ResMut<Clock>, mut motion: ResMut<Motion>) {
    let step = time.step.min(LONGEST_STEP);
    clock.since += step;
    clock.running += step;
    let since = clock.since;
    let swung = |share: f32| {
        let held = share.clamp(0.0, 1.0) - 1.0;
        1.0 + held * held * ((BACK_SWING + 1.0) * held + BACK_SWING)
    };
    let away = 1.0 - swung(since / SLIDE_TIME);
    let mut bars = [Span::Fixed(0.0); 3];
    for (place, share) in BAR_SHARES.iter().enumerate() {
        let start = BAR_DELAY + BAR_STAGGER * place as f32;
        let grown = swung((since - start) / BAR_TIME) * share;
        bars[place] = Span::Fixed(grown * BAR_TRACK);
    }
    let wanted = Motion {
        left: [-SLIDE_WIDTH * away, 0.0],
        right: [SLIDE_WIDTH * away, 0.0],
        stamp: ((since - STAMP_DELAY) / STAMP_TIME).clamp(0.0, 1.0),
        pulse: 0.55 + 0.45 * (clock.running * PULSE_SPEED).sin(),
        speed: bars[0],
        focus: bars[1],
        flair: bars[2],
    };
    if *motion != wanted {
        *motion = wanted;
    }
}
