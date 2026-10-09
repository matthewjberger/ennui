use crate::data::Ranges;
use crate::queries::range::{place, ranged_at};
use ennui::prelude::{Entity, Mut, Peek, Res, each_mut, peek, peek_mut};
use ennui_platform::prelude::{Input, MouseButton};
use ennui_ui::prelude::{Float, Hosted, Hosts, Panel, Rect, Span, pointer_in, renew, widen};

use ennui_ui_controls::prelude::LEAST_FILL;
use std::collections::HashMap;

pub(crate) fn drag_ranges(
    mut ranges: Ranges,
    mut panels: Mut<(Panel,)>,
    mut floats: Mut<(Float,)>,
    rects: Peek<Rect>,
    seats: Peek<Hosted>,
    hosts: Res<Hosts>,
    input: Res<Input>,
) {
    let began = input.buttons_pressed.contains(&MouseButton::Left);
    let down = input.buttons_held.contains(&MouseButton::Left);
    let mut spans: HashMap<Entity, Span> = HashMap::new();
    let mut spots: HashMap<Entity, Float> = HashMap::new();
    each_mut(
        &mut ranges,
        |entity, (mut held,), (rect, press, parts, grips)| {
            let at = pointer_in(&hosts, peek(&seats, entity));
            let ranged = *held;
            let wanted = ranged_at(ranged, rect, press.0, began, down, at);
            if wanted != ranged {
                *held = wanted;
            }
            let Some(parts) = parts else {
                return;
            };
            spans.insert(parts.before, Span::Fill(wanted.low.max(LEAST_FILL)));
            spans.insert(
                parts.middle,
                Span::Fill((wanted.high - wanted.low).max(LEAST_FILL)),
            );
            spans.insert(parts.after, Span::Fill((1.0 - wanted.high).max(LEAST_FILL)));
            let Some(grips) = grips else {
                return;
            };
            spots.insert(grips.low, place(&rects, grips.low, rect, wanted.low));
            spots.insert(grips.high, place(&rects, grips.high, rect, wanted.high));
        },
    );
    for (entity, next) in spans {
        if let Some((mut stamp,)) = peek_mut(&mut panels, entity) {
            widen(&mut stamp, next);
        }
    }
    for (entity, next) in spots {
        if let Some((mut stamp,)) = peek_mut(&mut floats, entity) {
            renew(&mut stamp, next);
        }
    }
}
