use crate::data::Bar;
use crate::theme::{
    ASPECT_TIP, NO_SCREENS, SCREEN_TIP, SHEET_DROP, SHEET_LAYER, SHEET_NOTE, SHEET_TITLE,
    SHEET_WIDE, SLOTS, SWATCH,
};
use editor_core::prelude::ASPECTS;
use ennui::later::{attach, set};
use ennui::prelude::{Entity, Later};
use ennui_ui::prelude::{
    Dye, Focus, Frame, Hidden, Host, Line, Lit, Off, Pin, Span, Theme, label, panel, pinned, sheet,
};
use ennui_ui_controls::prelude::{
    button, card_frame, checkbox, dim, field, hint, selectable, slider, small, tabs, toggle,
};

pub(crate) fn lay_bar(
    later: &mut Later,
    look: &Theme,
    (top, listing): (Entity, Entity),
    screens: &[String],
) -> Bar {
    let designing = panel(
        later,
        top,
        Frame::row(look)
            .wide(Span::Hug)
            .across(Line::Middle)
            .bare()
            .pad(0.0)
            .gap(look.gap * 0.5),
    );
    let chips = ASPECTS
        .iter()
        .map(|(name, _)| {
            let chip = button(later, look, designing, name);
            hint(later, chip, ASPECT_TIP)
        })
        .collect();
    let screens_row = panel(
        later,
        listing,
        Frame::column(look)
            .wide(Span::Fill(1.0))
            .bare()
            .pad(0.0)
            .gap(look.gap * 0.5),
    );
    if screens.is_empty() {
        let said = small(later, look, screens_row, NO_SCREENS);
        dim(later, said);
    }
    let screens = screens
        .iter()
        .map(|name| {
            let chip = button(later, look, screens_row, name);
            hint(later, chip, SCREEN_TIP);
            (chip, name.clone())
        })
        .collect();
    Bar {
        designing,
        chips,
        screens,
        screens_row,
    }
}

pub(crate) fn lay_sheet(later: &mut Later, look: &Theme, theme: Option<Entity>) -> Entity {
    let host = sheet(later, look, SHEET_LAYER);
    set(
        later,
        host,
        Host {
            layer: SHEET_LAYER,
            theme,
            ..Host::default()
        },
    );
    let card = pinned(
        later,
        host,
        card_frame(look, SHEET_WIDE).fill(Dye::Panel),
        Pin {
            at: [0.5, 0.0],
            nudge: [0.0, SHEET_DROP],
            pivot: [0.5, 0.0],
        },
    );
    label(later, look, card, SHEET_TITLE, look.heading);
    let note = small(later, look, card, SHEET_NOTE);
    dim(later, note);
    let row = |later: &mut Later| {
        panel(
            later,
            card,
            Frame::row(look).across(Line::Middle).bare().pad(0.0),
        )
    };
    let words = row(later);
    label(later, look, words, "Heading", look.heading);
    label(later, look, words, "Text", look.text);
    small(later, look, words, "Caption");
    let buttons = row(later);
    button(later, look, buttons, "Button");
    let lit = button(later, look, buttons, "Lit");
    set(later, lit, Lit(true));
    let focused = button(later, look, buttons, "Focus");
    set(later, focused, Focus(true));
    let off = button(later, look, buttons, "Off");
    set(later, off, Off(true));
    let switches = row(later);
    toggle(later, look, switches, "On", true);
    toggle(later, look, switches, "Off", false);
    checkbox(later, look, switches, "Checked", true);
    checkbox(later, look, switches, "Clear", false);
    let values = row(later);
    slider(later, look, values, 0.6);
    field(later, look, values, "Entry", "", 24);
    let rows = row(later);
    selectable(later, look, rows, "Chosen row", true);
    selectable(later, look, rows, "Row", false);
    tabs(later, look, card, &["One", "Two", "Three"], (0, 3));
    let swatches = row(later);
    for (name, dye) in SLOTS {
        let holder = panel(
            later,
            swatches,
            Frame::new(look).bare().pad(0.0).gap(look.gap * 0.25),
        );
        panel(
            later,
            holder,
            Frame::new(look)
                .wide(Span::Fixed(SWATCH))
                .tall(Span::Fixed(SWATCH))
                .fill(dye)
                .border(look.line)
                .rim(Dye::Edge)
                .pad(0.0),
        );
        let said = small(later, look, holder, name);
        dim(later, said);
    }
    attach(later, host, (Hidden(false),));
    host
}
