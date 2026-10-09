use crate::components::{Rank, Spun};
use crate::theme::{NOTE_DOT, NOTE_DOTS, NOTE_GAP};
use ennui::later::set;
use ennui::prelude::{Entity, Later};
use ennui_ui::prelude::{Dye, Frame, Lay, Span, Theme, label, panel, touched};

use ennui_ui_controls::prelude::{plain, small};

pub fn spinner(later: &mut Later, look: &Theme, parent: Entity) -> Entity {
    let held = panel(
        later,
        parent,
        Frame::new(look)
            .flow(Lay::Row)
            .wide(Span::Hug)
            .bare()
            .pad(0.0)
            .gap(NOTE_DOT * NOTE_GAP),
    );
    for place in 0..NOTE_DOTS {
        let dot = panel(
            later,
            held,
            Frame::new(look)
                .wide(Span::Fixed(NOTE_DOT))
                .tall(Span::Fixed(NOTE_DOT))
                .role(Dye::Accent)
                .round(NOTE_DOT * 0.5)
                .pad(0.0),
        );
        set(later, dot, Rank(place));
    }
    set(later, held, Spun);
    held
}

pub fn breadcrumb(later: &mut Later, look: &Theme, parent: Entity, path: &[&str]) -> Entity {
    let held = panel(later, parent, plain(Frame::row(look), 0.5));
    for (place, name) in path.iter().enumerate() {
        if place > 0 {
            small(later, look, held, "/");
        }
        let step = label(later, look, held, name, look.text);
        touched(later, step);
    }
    held
}
