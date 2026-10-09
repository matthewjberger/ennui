use super::press::button;
use super::scrub::scrub;
use super::text::small;
use crate::components::{Band, Channel, Dial, Dot, Knob, Mixed, Swatch, Wheel};
use crate::data::{Blend, RGB_NAMES};
use crate::queries::shape::{edged, plain};
use crate::queries::tint::hue_of;
use crate::theme::{PICKER_DOT, PICKER_STEP, PICKER_SWATCH, PICKER_WHEEL, TIGHT};
use ennui::later::{attach, set};
use ennui::prelude::{Entity, Later};
use ennui_platform::prelude::CursorIcon;
use ennui_ui::prelude::{
    Cursor, Dye, Effect, Frame, Line, Span, Theme, Tone, afloat, frame, panel, touched,
};

use nalgebra_glm::Vec4;

pub fn picker(later: &mut Later, look: &Theme, parent: Entity, color: Vec4) -> Entity {
    let held = panel(later, parent, plain(Frame::row(look), 1.0));
    let disc = panel(
        later,
        held,
        Frame::new(look)
            .wide(Span::Fixed(PICKER_WHEEL))
            .tall(Span::Fixed(PICKER_WHEEL))
            .fill(Vec4::repeat(1.0))
            .round(PICKER_WHEEL * 0.5)
            .pad(0.0),
    );
    let (hue, saturation, value) = hue_of(color.xyz());
    touched(later, disc);
    attach(
        later,
        disc,
        (
            Effect([1.0, value, 0.0, 0.0]),
            Wheel { hue, saturation },
            Band(held),
            Cursor(CursorIcon::Crosshair),
        ),
    );
    let dot = frame(
        later,
        edged(Frame::new(look))
            .wide(Span::Fixed(PICKER_DOT))
            .tall(Span::Fixed(PICKER_DOT))
            .role(Dye::Ink)
            .round(PICKER_DOT * 0.5)
            .pad(0.0),
    );
    afloat(later, held, dot);
    set(later, disc, Dot(dot));
    let column = panel(later, held, plain(Frame::column(look), 0.5));
    let shown = panel(
        later,
        column,
        edged(Frame::new(look))
            .wide(Span::Fixed(PICKER_SWATCH))
            .tall(Span::Fixed(PICKER_SWATCH))
            .fill(color)
            .pad(0.0),
    );
    set(later, shown, Tone::filled(color));
    let bars = panel(
        later,
        column,
        plain(
            Frame::new(look).wide(Span::Fill(1.0)).along(Line::Start),
            TIGHT,
        ),
    );
    for (place, name) in RGB_NAMES.iter().enumerate() {
        let line = panel(later, bars, plain(Frame::row(look), 0.5));
        let shown = small(later, look, line, name);
        attach(later, shown, (Channel(place), Band(held)));
        let bar = scrub(later, look, line, color[place], PICKER_STEP, (0.0, 1.0));
        attach(later, bar, (Channel(place), Band(held)));
    }
    let mode = button(later, look, column, "RGB");
    set(later, mode, Band(held));
    attach(
        later,
        held,
        (
            Mixed(Blend::Rgb),
            Knob(mode),
            Swatch(shown),
            Dial(disc),
            Tone::filled(color),
        ),
    );
    held
}
