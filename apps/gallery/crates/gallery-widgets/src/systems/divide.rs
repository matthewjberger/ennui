use crate::commands::divide::{set_span, span_of};
use crate::components::Divider;
use crate::theme::{LEAST_SHARE, MOST_SHARE};
use ennui::prelude::{Mut, Peek, Res, View, each, peek};
use ennui_platform::prelude::Input;
use ennui_ui::prelude::{Hosted, Hosts, Panel, Press, Rect, Span};

pub(crate) fn drag_dividers(
    dividers: View<(&Divider, &Press, Option<&Hosted>)>,
    rects: Peek<Rect>,
    mut panels: Mut<(Panel,)>,
    hosts: Res<Hosts>,
    input: Res<Input>,
) {
    let across = input.pointer_motion[0];
    let down = input.pointer_motion[1];
    if across == 0.0 && down == 0.0 {
        return;
    }
    let held: Vec<(Divider, f32)> = each(&dividers)
        .filter(|(_, (_, press, _))| press.0)
        .filter_map(|(_, (divider, _, hosted))| {
            let unit = 1.0
                / hosted
                    .and_then(|hosted| hosts.list.iter().find(|held| held.host == hosted.0))
                    .map_or(1.0, |held| held.pixels.max(f32::EPSILON));
            let first = peek(&rects, divider.first)?.size;
            let second = peek(&rects, divider.second)?.size;
            let (moved, room) = match divider.down {
                true => (down, first.y + second.y),
                false => (across, first.x + second.x),
            };
            Some((*divider, moved * unit / room.max(f32::EPSILON)))
        })
        .collect();
    for (divider, step) in held {
        let Some(Span::Fill(first)) = span_of(&mut panels, divider.first, divider.down) else {
            continue;
        };
        let wanted = (first + step).clamp(LEAST_SHARE, MOST_SHARE);
        set_span(&mut panels, divider.first, divider.down, wanted);
        set_span(&mut panels, divider.second, divider.down, 1.0 - wanted);
    }
}
