use super::text::small;
use crate::queries::shape::plain;
use ennui::prelude::{Entity, Later};
use ennui_ui::prelude::{Frame, Line, Span, Theme, label, panel};

pub fn stat(later: &mut Later, look: &Theme, parent: Entity, name: &str, value: &str) -> Entity {
    let row = panel(later, parent, plain(Frame::row(look), 0.5));
    small(later, look, row, name);
    let rest = rest_of(later, look, row);
    label(later, look, rest, value, look.text)
}

pub fn rest_of(later: &mut Later, look: &Theme, row: Entity) -> Entity {
    panel(
        later,
        row,
        Frame::new(look)
            .wide(Span::Fill(1.0))
            .across(Line::End)
            .bare()
            .pad(0.0),
    )
}
