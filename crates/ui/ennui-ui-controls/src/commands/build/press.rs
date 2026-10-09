use crate::components::{Band, Button, Knob, Marked, Skinned, Swing, Toggle, Track};
use crate::queries::shape::{
    box_of, mark_frame, row_of, strip, switch_knob, switch_track, wide_row,
};
use crate::theme::{CHECK_INSET, RADIO_INSET};
use ennui::later::{attach, set};
use ennui::prelude::{Entity, Later};
use ennui_ui::prelude::{Dye, Lit, Theme, afloat, frame, label, panel, shortened, touched};

pub fn button(later: &mut Later, look: &Theme, parent: Entity, text: &str) -> Entity {
    let held = panel(
        later,
        parent,
        wide_row(look).role(Dye::Ground).pad(look.pad * 0.5),
    );
    label(later, look, held, text, look.text);
    attach(later, held, (Button::default(), Skinned));
    touched(later, held)
}

pub fn checkbox(later: &mut Later, look: &Theme, parent: Entity, text: &str, on: bool) -> Entity {
    let held = panel(later, parent, row_of(look));
    let outer = panel(later, held, box_of(look, CHECK_INSET));
    let mark = panel(later, outer, mark_frame(look, look.round * 0.5));
    set(later, mark, Lit(on));
    label(later, look, held, text, look.text);
    attach(later, held, (Toggle(on), Knob(mark), Skinned));
    touched(later, held)
}

pub fn toggle(later: &mut Later, look: &Theme, parent: Entity, text: &str, on: bool) -> Entity {
    let held = panel(later, parent, row_of(look));
    let track = panel(later, held, switch_track(look));
    let knob = frame(later, switch_knob(look));
    afloat(later, track, knob);
    label(later, look, held, text, look.text);
    attach(
        later,
        held,
        (
            Toggle(on),
            Swing(f32::from(on)),
            Knob(knob),
            Track(track),
            Skinned,
        ),
    );
    touched(later, held)
}

pub fn radio(
    later: &mut Later,
    look: &Theme,
    parent: Entity,
    band: Entity,
    text: &str,
    on: bool,
) -> Entity {
    let held = panel(later, parent, row_of(look));
    let ring = panel(
        later,
        held,
        box_of(look, RADIO_INSET).round(look.box_size * 0.5),
    );
    let dot = panel(later, ring, mark_frame(look, look.box_size * 0.5));
    set(later, dot, Lit(on));
    label(later, look, held, text, look.text);
    attach(later, held, (Toggle(on), Knob(dot), Band(band), Skinned));
    touched(later, held)
}

pub fn selectable(later: &mut Later, look: &Theme, parent: Entity, text: &str, on: bool) -> Entity {
    let held = panel(later, parent, strip(look));
    shortened(later, look, held, text, look.text);
    attach(later, held, (Lit(on), Marked(on)));
    touched(later, held)
}
