use super::value::{caret_of, swath_of, typable};
use crate::commands::edit::edit_of;
use crate::components::{Field, Knob, Scrub};
use crate::data::MIXED;
use crate::queries::scrub::scrub_label;
use crate::queries::shape::input_frame;
use crate::theme::SCRUB_ROOM;
use ennui::later::{change, set};
use ennui::prelude::{Edits, Entity, Later, Storage};
use ennui::storage::get;
use ennui_platform::prelude::CursorIcon;
use ennui_text::prelude::Label;
use ennui_ui::prelude::{Line, Press, Theme, label, panel};

pub fn scrub(
    later: &mut Later,
    look: &Theme,
    parent: Entity,
    value: f32,
    step: f32,
    span: (f32, f32),
) -> Entity {
    let held = panel(later, parent, input_frame(look).along(Line::Middle));
    let band = swath_of(later, look, held);
    let text = scrub_label(value);
    let shown = label(later, look, held, &text, look.text);
    let bar = caret_of(later, look, held);
    set(
        later,
        held,
        Scrub {
            value,
            step,
            low: span.0,
            high: span.1,
            moved: 0.0,
            typing: false,
        },
    );
    typable(
        later,
        held,
        &text,
        SCRUB_ROOM,
        [shown, bar, band],
        CursorIcon::EwResize,
    )
}

pub fn put_scrub(seen: &Storage, edits: &mut Edits, entity: Entity, value: Option<f32>) -> bool {
    let free = get::<Scrub>(seen, entity).is_some_and(|scrub| !scrub.typing)
        && !get::<Press>(seen, entity).is_some_and(|held| held.0);
    if free {
        scrub_to(edits, entity, value);
    }
    free
}

pub(crate) fn scrub_to(edits: &mut Edits, entity: Entity, value: Option<f32>) {
    change(edits, move |storage| {
        let Some(scrub) = get::<Scrub>(&*storage, entity).copied() else {
            return;
        };
        let pressed = get::<Press>(&*storage, entity).is_some_and(|held| held.0);
        if scrub.typing || pressed {
            return;
        }
        let knob = get::<Knob>(&*storage, entity).map(|held| held.0);
        let text = match value {
            Some(value) => {
                let text = scrub_label(value);
                ennui::storage::attach(
                    &mut *storage,
                    entity,
                    (
                        Scrub { value, ..scrub },
                        edit_of(&text),
                        Field(text.clone()),
                    ),
                );
                text
            }
            None => String::from(MIXED),
        };
        if let Some(knob) = knob {
            ennui::storage::set_if_new(&mut *storage, knob, Label(text));
        }
    });
}
