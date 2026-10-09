use crate::resources::Display;
use crate::theme::SCRGB_NITS;

pub fn white(display: &Display) -> f32 {
    match display.wide {
        true => display.paper.max(1.0) / SCRGB_NITS,
        false => 1.0,
    }
}
