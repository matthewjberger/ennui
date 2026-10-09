use crate::queries::shape::header_frame;
use crate::theme::{RULE_TALL_SHARE, SPACING_SHARE};
use ennui::prelude::{Edits, Entity, Later};
use ennui_ui::prelude::{Dye, Frame, Line, Span, Theme, ink, label, panel, touched};

use ennui_ui_icons::prelude::icon;

pub fn dim(edits: &mut Edits, entity: Entity) -> Entity {
    ink(edits, entity, Dye::Faint, Dye::Ink);
    entity
}

pub fn knob(later: &mut Later, look: &Theme, parent: Entity, glyph: char) -> (Entity, Entity) {
    let held = panel(
        later,
        parent,
        Frame::new(look)
            .wide(Span::Fixed(look.row))
            .tall(Span::Fixed(look.row))
            .roles(Dye::Ground, Dye::Accent)
            .border(look.line)
            .rim(Dye::Edge)
            .pad(0.0)
            .gap(0.0),
    );
    let shown = icon(later, look, held, glyph, look.text);
    (touched(later, held), shown)
}

pub fn spread(later: &mut Later, look: &Theme, parent: Entity) -> Entity {
    panel(
        later,
        parent,
        Frame::new(look)
            .wide(Span::Fill(1.0))
            .bare()
            .pad(0.0)
            .gap(0.0),
    )
}

pub fn rule(later: &mut Later, look: &Theme, parent: Entity) -> Entity {
    panel(
        later,
        parent,
        Frame::new(look)
            .wide(Span::Fixed(look.line))
            .tall(Span::Fixed(look.text * RULE_TALL_SHARE))
            .role(Dye::Edge)
            .round(0.0)
            .pad(0.0)
            .gap(0.0),
    )
}

pub fn holder(later: &mut Later, look: &Theme, parent: Entity, wide: f32) -> Entity {
    panel(
        later,
        parent,
        Frame::new(look)
            .wide(Span::Fixed(wide))
            .tall(Span::Fill(1.0))
            .along(Line::Start)
            .bare()
            .pad(look.pad)
            .gap(0.0),
    )
}

pub fn desk(later: &mut Later, look: &Theme, parent: Entity) -> Entity {
    panel(
        later,
        parent,
        Frame::new(look)
            .wide(Span::Fill(1.0))
            .tall(Span::Fill(1.0))
            .along(Line::Start)
            .role(Dye::Panel)
            .border(look.line)
            .rim(Dye::Edge)
            .lifted()
            .pad(0.0)
            .gap(0.0),
    )
}

pub fn top_bar(later: &mut Later, look: &Theme, parent: Entity, tall: f32) -> Entity {
    panel(
        later,
        parent,
        header_frame(look, tall)
            .pad(look.pad)
            .gap(look.gap * SPACING_SHARE),
    )
}

pub fn foot_bar(later: &mut Later, look: &Theme, parent: Entity, tall: f32) -> Entity {
    let foot = panel(
        later,
        parent,
        header_frame(look, tall).pad(look.pad).gap(look.gap * 0.5),
    );
    let hint = label(later, look, foot, "", look.text);
    dim(later, hint)
}

pub fn head(later: &mut Later, look: &Theme, parent: Entity, glyph: char, text: &str) -> Entity {
    let bar = panel(
        later,
        parent,
        header_frame(look, look.header_tall)
            .pad(look.pad * SPACING_SHARE)
            .gap(look.gap * 0.5),
    );
    icon(later, look, bar, glyph, look.text);
    label(later, look, bar, text, look.text);
    bar
}
