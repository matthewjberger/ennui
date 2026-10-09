use crate::queries::keys::chords;
use crate::theme::{SCRUB_FAST, SCRUB_SLOW};
use ennui_platform::prelude::{Input, KeyCode};

pub(crate) fn scrub_label(value: f32) -> String {
    format!("{value:.3}")
}

pub(crate) fn pace_of(input: &Input) -> f32 {
    let (_, fast) = chords(input);
    let slow = input.held.contains(&KeyCode::AltLeft) || input.held.contains(&KeyCode::AltRight);
    match (fast, slow) {
        (true, _) => SCRUB_FAST,
        (false, true) => SCRUB_SLOW,
        (false, false) => 1.0,
    }
}
