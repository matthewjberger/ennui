use ennui::prelude::{Entity, Peek, peek};
use ennui_text::prelude::{Family, Height, Wrap};

pub(crate) fn lettering(
    (heights, families): (&Peek<Height>, &Peek<Family>),
    knob: Entity,
    wrap: Option<&Wrap>,
) -> (f32, f32, String) {
    (
        peek(heights, knob).map_or(0.0, |held| held.0),
        wrap.map_or(0.0, |held| held.0),
        peek(families, knob).map_or(String::new(), |held| held.0.clone()),
    )
}
