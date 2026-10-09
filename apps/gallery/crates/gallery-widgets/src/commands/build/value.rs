use crate::theme::LIST_ROWS;
use ennui::prelude::{Entity, Later};
use ennui_ui::prelude::{Dye, Frame, Line, Span, Theme, scroll};

use ennui_ui_controls::prelude::{SNUG, edged, selectable};

pub fn multi(later: &mut Later, look: &Theme, parent: Entity, options: &[&str]) -> Entity {
    let held = scroll(
        later,
        look,
        parent,
        edged(Frame::new(look))
            .wide(Span::Fill(1.0))
            .tall(Span::Fixed(look.row * LIST_ROWS))
            .along(Line::Start)
            .role(Dye::Input)
            .pad(look.pad * SNUG)
            .gap(2.0),
    );
    for text in options {
        selectable(later, look, held, text, false);
    }
    held
}
