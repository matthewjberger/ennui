use crate::data::Source;
use crate::theme::STEP_REACH;
use ennui_platform::prelude::Input;
use nalgebra_glm::Vec2;

pub(crate) fn sourced(mut wanted: Vec2, source: &Source, input: &Input) -> (Vec2, bool) {
    let mut pressed = false;
    match source {
        Source::Key(key) => {
            if input.held.contains(key) {
                wanted.x = 1.0;
            }
            pressed = input.pressed.contains(key);
        }
        Source::Pointer(button) => {
            if input.buttons_held.contains(button) {
                wanted.x = 1.0;
            }
            pressed = input.buttons_pressed.contains(button);
        }
        Source::Pair { less, more } => {
            wanted.x += f32::from(input.held.contains(more)) - f32::from(input.held.contains(less));
        }
        Source::Cross {
            less,
            more,
            down,
            up,
        } => {
            wanted.x += f32::from(input.held.contains(more)) - f32::from(input.held.contains(less));
            wanted.y += f32::from(input.held.contains(up)) - f32::from(input.held.contains(down));
        }
        Source::Wheel => {
            wanted.x += input.scroll;
        }
    }
    (wanted, pressed)
}

pub(crate) fn snapped(leaned: Vec2) -> Option<Vec2> {
    if leaned.norm() < STEP_REACH {
        return None;
    }
    Some(match leaned.x.abs() >= leaned.y.abs() {
        true => Vec2::new(leaned.x.signum(), 0.0),
        false => Vec2::new(0.0, leaned.y.signum()),
    })
}

pub(crate) fn capped(wanted: Vec2) -> Vec2 {
    let reach = wanted.norm();
    match reach > 1.0 {
        true => wanted / reach,
        false => wanted,
    }
}
