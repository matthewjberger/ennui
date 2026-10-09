use crate::resources::Rate;
use ennui::prelude::{Entity, Later};
use ennui_ui::prelude::{Line, Pin, Theme, pinned};

use ennui_ui_controls::prelude::{card_frame, small};

pub fn lay_rate(later: &mut Later, look: &Theme, rate: &mut Rate, parent: Entity) {
    let share = |line: Line| match line {
        Line::Start => 0.0,
        Line::Middle => 0.5,
        Line::End => 1.0,
    };
    let at = [share(rate.along), share(rate.across)];
    let pin = Pin {
        at,
        nudge: at.map(|held| rate.margin * (1.0 - 2.0 * held)),
        pivot: at,
    };
    let card = pinned(later, parent, card_frame(look, rate.width), pin);
    rate.card = Some(card);
    rate.label = Some(small(later, look, card, "-"));
}
