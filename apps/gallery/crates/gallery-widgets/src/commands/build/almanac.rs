use crate::components::{Dated, Rank, Shifted, Stack};
use crate::data::DAYS;
use crate::queries::calendar::month_title;
use crate::theme::{ALMANAC_CELL, ALMANAC_HEAD, ALMANAC_WEEKS};
use ennui::later::attach;
use ennui::prelude::{Entity, Later};
use ennui_ui::prelude::{Dye, Frame, Lay, Line, Lit, Span, Theme, label, panel, touched};

use ennui_ui_controls::prelude::{Band, Knob, SNUG, button, edged, plain, small};

pub fn almanac(later: &mut Later, look: &Theme, parent: Entity, held: Dated) -> Entity {
    let card = panel(
        later,
        parent,
        edged(Frame::new(look))
            .wide(Span::Hug)
            .along(Line::Start)
            .role(Dye::Panel)
            .pad(look.pad * 0.5)
            .gap(look.gap * SNUG),
    );
    let head = panel(
        later,
        card,
        plain(Frame::new(look).flow(Lay::Row).wide(Span::Fill(1.0)), 0.5),
    );
    let back = button(later, look, head, "<");
    attach(later, back, (Shifted(-1), Band(card)));
    let shown = label(later, look, head, &month_title(held), look.text);
    let ahead = button(later, look, head, ">");
    attach(later, ahead, (Shifted(1), Band(card)));
    let week = panel(
        later,
        card,
        Frame::new(look).flow(Lay::Row).bare().pad(0.0).gap(0.0),
    );
    for name in DAYS.iter() {
        let cell = panel(
            later,
            week,
            Frame::new(look)
                .wide(Span::Fixed(ALMANAC_CELL))
                .tall(Span::Fixed(ALMANAC_CELL * ALMANAC_HEAD))
                .bare()
                .pad(0.0),
        );
        small(later, look, cell, name);
    }
    let grid = panel(
        later,
        card,
        Frame::new(look).along(Line::Start).bare().pad(0.0).gap(0.0),
    );
    for week in 0..ALMANAC_WEEKS {
        let line = panel(
            later,
            grid,
            Frame::new(look).flow(Lay::Row).bare().pad(0.0).gap(0.0),
        );
        for day in 0..7 {
            let cell = panel(
                later,
                line,
                Frame::new(look)
                    .wide(Span::Fixed(ALMANAC_CELL))
                    .tall(Span::Fixed(ALMANAC_CELL))
                    .roles(Dye::Panel, Dye::Accent)
                    .round(look.round * 0.5)
                    .pad(0.0),
            );
            label(later, look, cell, " ", look.text);
            attach(
                later,
                cell,
                (Rank((week * 7 + day) as usize), Band(card), Lit(false)),
            );
            touched(later, cell);
        }
    }
    attach(later, card, (held, Knob(shown), Stack(grid)));
    card
}
