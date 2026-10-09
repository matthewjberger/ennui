use crate::components::Divider;
use crate::theme::{LEAST_SHARE, MOST_SHARE, SPLIT_BAR};
use ennui::later::set;
use ennui::prelude::{Entity, Later};
use ennui_platform::prelude::CursorIcon;
use ennui_ui::prelude::{Cursor, Dye, Frame, Lay, Span, Theme, panel, touched};

pub fn split(
    later: &mut Later,
    look: &Theme,
    parent: Entity,
    flow: Lay,
    share: f32,
) -> (Entity, Entity, Entity) {
    let held = panel(
        later,
        parent,
        Frame::new(look)
            .flow(flow)
            .wide(Span::Fill(1.0))
            .tall(Span::Fill(1.0))
            .bare()
            .pad(0.0)
            .gap(0.0),
    );
    let first = panel(
        later,
        held,
        span(look, flow, share.clamp(LEAST_SHARE, MOST_SHARE)),
    );
    let bar = panel(
        later,
        held,
        match flow {
            Lay::Row => Frame::new(look)
                .wide(Span::Fixed(SPLIT_BAR))
                .tall(Span::Fill(1.0)),
            Lay::Column => Frame::new(look)
                .wide(Span::Fill(1.0))
                .tall(Span::Fixed(SPLIT_BAR)),
        }
        .role(Dye::Edge)
        .round(0.0)
        .pad(0.0),
    );
    let second = panel(
        later,
        held,
        span(look, flow, (1.0 - share).clamp(LEAST_SHARE, MOST_SHARE)),
    );
    set(
        later,
        bar,
        Divider {
            first,
            second,
            down: matches!(flow, Lay::Column),
        },
    );
    touched(later, bar);
    let icon = match flow {
        Lay::Row => CursorIcon::EwResize,
        Lay::Column => CursorIcon::NsResize,
    };
    set(later, bar, Cursor(icon));
    (first, second, bar)
}

fn span(look: &Theme, flow: Lay, share: f32) -> Frame<'_> {
    let held = Frame::new(look).bare().pad(0.0);
    match flow {
        Lay::Row => held.wide(Span::Fill(share)).tall(Span::Fill(1.0)),
        Lay::Column => held.wide(Span::Fill(1.0)).tall(Span::Fill(share)),
    }
}
