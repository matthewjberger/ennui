use crate::theme::{
    CARD_PAD, KNOB_PAD, KNOB_SOFT, LIST_GAP, LOOSE, ROW_TEXT, TIGHT, TOGGLE_DROP, TRACK_SOFT,
};
use ennui_ui::prelude::{Dye, Frame, Lay, Line, Span, Theme};

pub fn plain(held: Frame<'_>, share: f32) -> Frame<'_> {
    let look = held.theme;
    held.bare().pad(0.0).gap(look.gap * share)
}

pub fn filling(look: &Theme) -> Frame<'_> {
    Frame::new(look)
        .wide(Span::Fill(1.0))
        .tall(Span::Fill(1.0))
        .along(Line::Start)
        .bare()
        .pad(0.0)
}

pub fn edged(held: Frame<'_>) -> Frame<'_> {
    let look = held.theme;
    held.border(look.line).rim(Dye::Edge)
}

pub(crate) fn wide_row(look: &Theme) -> Frame<'_> {
    edged(Frame::new(look))
        .wide(Span::Fill(1.0))
        .tall(Span::Fixed(look.row))
}

pub(crate) fn row_of(look: &Theme) -> Frame<'_> {
    plain(
        Frame::row(look).tall(Span::Fixed(look.text * ROW_TEXT)),
        1.0,
    )
}

pub(crate) fn box_of(look: &Theme, inset: f32) -> Frame<'_> {
    edged(Frame::new(look))
        .wide(Span::Fixed(look.box_size))
        .tall(Span::Fixed(look.box_size))
        .role(Dye::Input)
        .pad(inset)
}

pub(crate) fn strip(look: &Theme) -> Frame<'_> {
    Frame::row(look)
        .tall(Span::Fixed(look.row))
        .roles(Dye::Panel, Dye::Chosen)
        .pad(look.pad * LOOSE)
        .gap(look.gap)
}

pub fn card_frame(look: &Theme, wide: f32) -> Frame<'_> {
    Frame::new(look)
        .wide(Span::Fixed(wide))
        .along(Line::Start)
        .across(Line::Start)
        .fill(Dye::Panel)
        .border(look.line)
        .pad(look.pad * CARD_PAD)
        .gap(look.gap)
}

pub(crate) fn list_frame(look: &Theme, wide: f32) -> Frame<'_> {
    edged(Frame::new(look))
        .wide(Span::Fixed(wide))
        .along(Line::Start)
        .role(Dye::Panel)
        .lifted()
        .pad(look.pad * 0.5)
        .gap(LIST_GAP)
}

pub(crate) fn input_frame(look: &Theme) -> Frame<'_> {
    edged(Frame::row(look))
        .tall(Span::Fixed(look.row))
        .role(Dye::Input)
        .pad(look.pad * LOOSE)
}

pub(crate) fn mark_frame(look: &Theme, round: f32) -> Frame<'_> {
    Frame::new(look)
        .wide(Span::Fill(1.0))
        .tall(Span::Fill(1.0))
        .roles(Dye::Input, Dye::Accent)
        .round(round)
        .pad(0.0)
}

pub(crate) fn switch_track(look: &Theme) -> Frame<'_> {
    Frame::new(look)
        .flow(Lay::Row)
        .along(Line::Start)
        .wide(Span::Fixed(look.toggle_wide))
        .tall(Span::Fixed(look.toggle_tall))
        .role(Dye::Ground)
        .round(look.toggle_tall * 0.5)
        .shade(Dye::Shade, [0.0, -TOGGLE_DROP, look.soft * TRACK_SOFT, 0.0])
        .pad(KNOB_PAD)
        .gap(0.0)
}

pub(crate) fn switch_knob(look: &Theme) -> Frame<'_> {
    Frame::new(look)
        .wide(Span::Fixed(look.toggle_tall - KNOB_PAD * 2.0))
        .tall(Span::Fixed(look.toggle_tall - KNOB_PAD * 2.0))
        .role(Dye::Ink)
        .round(look.toggle_tall * 0.5 - KNOB_PAD)
        .shade(Dye::Shade, [0.0, -TOGGLE_DROP, look.soft * KNOB_SOFT, 0.0])
        .pad(0.0)
}

pub(crate) fn track_frame(look: &Theme, tall: f32) -> Frame<'_> {
    Frame::new(look)
        .flow(Lay::Row)
        .wide(Span::Fill(1.0))
        .tall(Span::Fixed(tall))
        .role(Dye::Input)
        .pad(0.0)
        .gap(0.0)
}

pub(crate) fn header_frame(look: &Theme, tall: f32) -> Frame<'_> {
    Frame::row(look)
        .tall(Span::Fixed(tall))
        .role(Dye::Header)
        .round(0.0)
}

pub(crate) fn tab_bar(look: &Theme) -> Frame<'_> {
    Frame::row(look)
        .wide(Span::Fill(1.0))
        .role(Dye::Panel)
        .pad(look.pad * TIGHT)
        .gap(look.gap * TIGHT)
}
