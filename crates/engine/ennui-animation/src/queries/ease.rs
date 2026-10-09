use crate::components::Play;
use crate::data::{
    BACK_OVERSHOOT, BOUNCE_PULL, BOUNCES, Bound, EPSILON, Ease, Form, LANES, Strand,
};
use ennui::reflect::prelude::Value;

pub(crate) fn eased(ease: Ease, share: f32) -> f32 {
    let share = share.clamp(0.0, 1.0);
    match ease {
        Ease::Linear => share,
        Ease::Step => 0.0,
        Ease::In => share * share * share,
        Ease::Out => 1.0 - (1.0 - share).powi(3),
        Ease::InOut => match share < 0.5 {
            true => 4.0 * share * share * share,
            false => 1.0 - (2.0 - 2.0 * share).powi(3) / 2.0,
        },
        Ease::Smooth => share * share * (3.0 - 2.0 * share),
        Ease::Back => (BACK_OVERSHOOT + 1.0) * share.powi(3) - BACK_OVERSHOOT * share * share,
        Ease::Bounce => {
            let (_, shift, lift) = BOUNCES
                .iter()
                .find(|(edge, _, _)| share < *edge)
                .copied()
                .unwrap_or(BOUNCES[BOUNCES.len() - 1]);
            let shifted = share - shift;
            BOUNCE_PULL * shifted * shifted + lift
        }
    }
}

pub(crate) fn width_of(form: Form) -> usize {
    match form {
        Form::Numbers(width) => width,
        Form::Whole | Form::Flag => 1,
        Form::Other => 0,
    }
}

pub(crate) fn lanes_at(bound: &Bound, time: f32) -> [f64; LANES] {
    let (before, after, share) = span(&bound.times, time);
    let width = width_of(bound.writer.form);
    let part = f64::from(eased(
        bound.eases.get(before).copied().unwrap_or_default(),
        share,
    ));
    let mut held = [0.0; LANES];
    for (lane, value) in held.iter_mut().enumerate().take(width) {
        let from = bound.values.get(before * width + lane).copied();
        let to = bound.values.get(after * width + lane).copied();
        *value = match (from, to) {
            (Some(from), Some(to)) => from + (to - from) * part,
            (Some(from), None) => from,
            _ => 0.0,
        };
    }
    held
}

pub(crate) fn strand_of(play: &Play, looping: bool) -> Strand {
    Strand {
        time: play.time,
        last: play.last,
        looping,
    }
}

pub(crate) fn span(times: &[f32], at: f32) -> (usize, usize, f32) {
    if times.len() < 2 {
        return (0, 0, 0.0);
    }
    if at <= times[0] {
        return (0, 0, 0.0);
    }
    let last = times.len() - 1;
    if at >= times[last] {
        return (last, last, 0.0);
    }
    let after = times.partition_point(|held| *held <= at).min(last);
    let before = after - 1;
    let width = (times[after] - times[before]).max(EPSILON);
    (before, after, (at - times[before]) / width)
}

pub(crate) fn crossed(strand: Strand, at: f32, length: f32) -> bool {
    if strand.time == strand.last {
        return false;
    }
    if length <= 0.0 || !strand.looping {
        return strand.last < at && at <= strand.time;
    }
    let (last, now) = (
        strand.last.rem_euclid(length),
        strand.time.rem_euclid(length),
    );
    match now >= last {
        true => last < at && at <= now,
        false => at > last || at <= now,
    }
}

pub(crate) fn held_at(bound: &Bound, time: f32) -> Option<&Value> {
    let (before, _, _) = span(&bound.times, time);
    bound.held.get(before)
}
