use crate::components::{Head, Rank, Sorted, Stack};
use crate::theme::GRID_HEAD;
use ennui::later::{attach, change, set, within};
use ennui::prelude::{Edits, Entity, Later};
use ennui::storage::get;
use ennui_ui::prelude::{
    Dye, Frame, Line, Lit, Span, Theme, frame, ink, label, panel, scroll, spacing, touched, under,
};

use ennui_ui_controls::prelude::{edged, filling, plain, small};

pub fn grid(later: &mut Later, look: &Theme, parent: Entity, columns: &[(&str, f32)]) -> Entity {
    let held = panel(
        later,
        parent,
        edged(filling(look)).role(Dye::Panel).gap(0.0),
    );
    let head = panel(
        later,
        held,
        Frame::row(look)
            .tall(Span::Fixed(look.header_tall))
            .role(Dye::Header)
            .pad(look.pad * 0.5)
            .gap(0.0),
    );
    for (place, (text, share)) in columns.iter().enumerate() {
        let cell = panel(later, head, cell_frame(look, *share));
        let shown = small(later, look, cell, text);
        ink(later, shown, Dye::Faint, Dye::Accent);
        set(later, shown, Lit(place == 0));
        set(later, cell, Rank(place));
        touched(later, cell);
    }
    spacing(later, look, head, look.bar_wide);
    let body = scroll(later, look, held, filling(look).gap(0.0));
    attach(later, held, (Head(head), Stack(body)));
    set(
        later,
        held,
        Sorted {
            rank: 0,
            rising: true,
        },
    );
    held
}

pub fn stripe(later: &mut Later, look: &Theme, held: Entity, place: usize) -> Entity {
    let row = frame(
        later,
        Frame::row(look)
            .tall(Span::Fixed(look.row * GRID_HEAD))
            .roles(Dye::Panel, Dye::Header)
            .round(0.0)
            .pad(look.pad * 0.5)
            .gap(0.0),
    );
    under_stack(later, held, row);
    set(later, row, Lit(!place.is_multiple_of(2)));
    touched(later, row);
    let inner = panel(
        later,
        row,
        plain(Frame::row(look).tall(Span::Fill(1.0)), 0.0),
    );
    spacing(later, look, row, look.bar_wide);
    set(later, row, Stack(inner));
    row
}

pub fn cell(later: &mut Later, look: &Theme, row: Entity, text: &str, share: f32) -> Entity {
    let held = frame(later, cell_frame(look, share));
    under_stack(later, row, held);
    label(later, look, held, text, look.text);
    held
}

fn under_stack(edits: &mut Edits, holder: Entity, child: Entity) {
    change(edits, move |storage| {
        let parent = get::<Stack>(&*storage, holder).map_or(holder, |held| held.0);
        within(storage, |later| under(later, parent, child));
    });
}

fn cell_frame(look: &Theme, share: f32) -> Frame<'_> {
    Frame::new(look)
        .wide(Span::Fill(share))
        .tall(Span::Fill(1.0))
        .along(Line::Middle)
        .across(Line::Start)
        .bare()
        .pad(0.0)
}
