use crate::components::Ranged;
use ennui::prelude::{Entity, Peek, peek};
use ennui_ui::prelude::{Float, Rect};

use ennui_ui_controls::prelude::LEAST_FILL;
use nalgebra_glm::Vec2;

pub(crate) fn place(rects: &Peek<Rect>, grip: Entity, track: &Rect, share: f32) -> Float {
    let size = peek(rects, grip).map_or(Vec2::zeros(), |held| held.size);
    let middle = track.center.x - track.size.x * 0.5 + track.size.x * share;
    Float(Vec2::new(
        middle - size.x * 0.5,
        track.center.y + size.y * 0.5,
    ))
}

pub(crate) fn ranged_at(
    ranged: Ranged,
    rect: &Rect,
    pressed: bool,
    began: bool,
    down: bool,
    at: Vec2,
) -> Ranged {
    let mut wanted = ranged;
    if !down {
        wanted.grabbed = 0;
    }
    if pressed {
        let share = ((at.x - rect.center.x) / rect.size.x.max(LEAST_FILL) + 0.5).clamp(0.0, 1.0);
        if began {
            wanted.grabbed = match (share - ranged.low).abs() <= (share - ranged.high).abs() {
                true => 1,
                false => 2,
            };
        }
        match wanted.grabbed {
            1 => wanted.low = share.min(ranged.high),
            2 => wanted.high = share.max(ranged.low),
            _ => {}
        }
    }
    wanted
}
